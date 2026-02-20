//! MP4Box 转换核心
//! 支持新版 B 站 m4s 的 9 字节头部填充去除

use crate::cache::VideoInfo;
use crate::filemgr::{resolve_output_path, ConflictStrategy};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConvertProgress {
    pub current_file: String,
    pub current_index: usize,
    pub total: usize,
    pub percent: u32,
}

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("文件不存在: {0}")]
    FileNotFound(String),
    #[error("MP4Box 未找到，请安装: brew install gpac")]
    Mp4BoxNotFound,
    #[error("MP4Box 执行失败: {0}")]
    Mp4BoxFailed(String),
    #[error("用户取消")]
    Cancelled,
    #[error("冲突策略跳过")]
    Skipped(#[from] crate::filemgr::ConflictError),
}

const M4S_HEADER_PADDING: [u8; 9] = [0x30; 9];

struct TempCleanup(Option<PathBuf>, Option<PathBuf>);
impl TempCleanup {
    fn new(v: Option<PathBuf>, a: Option<PathBuf>) -> Self { Self(v, a) }
}
impl Drop for TempCleanup {
    fn drop(&mut self) {
        let _ = self.0.take().map(std::fs::remove_file);
        let _ = self.1.take().map(std::fs::remove_file);
    }
}

/// 若 m4s 含 9 字节 0x30 填充，去除后写入临时文件并返回路径；否则返回原路径
fn ensure_clean_m4s(path: &Path) -> Result<(PathBuf, bool), std::io::Error> {
    let mut f = File::open(path)?;
    let mut header = [0u8; 9];
    f.read_exact(&mut header)?;
    if header != M4S_HEADER_PADDING {
        return Ok((path.to_path_buf(), false));
    }
    let temp_dir = std::env::temp_dir().join("bili2mp4");
    std::fs::create_dir_all(&temp_dir)?;
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_path = temp_dir.join(format!(
        "{}_{}",
        suffix,
        path.file_name().and_then(|n| n.to_str()).unwrap_or("m4s")
    ));
    let mut out = File::create(&temp_path)?;
    std::io::copy(&mut f, &mut out)?;
    Ok((temp_path, true))
}

/// 转换单个视频（自动处理 9 字节头部）
pub fn convert_one(
    video: &VideoInfo,
    out_dir: &Path,
    mp4box_path: &str,
    strategy: ConflictStrategy,
    on_progress: impl Fn(ConvertProgress),
    cancel: &AtomicBool,
) -> Result<std::path::PathBuf, ConvertError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(ConvertError::Cancelled);
    }

    if !video.video_path.exists() {
        return Err(ConvertError::FileNotFound(
            video.video_path.display().to_string(),
        ));
    }
    if !video.audio_path.exists() {
        return Err(ConvertError::FileNotFound(
            video.audio_path.display().to_string(),
        ));
    }

    let output_path = resolve_output_path(out_dir, &video.title, strategy)?;

    on_progress(ConvertProgress {
        current_file: video.title.clone(),
        current_index: 1,
        total: 1,
        percent: 0,
    });

    let (video_clean, video_temp) = ensure_clean_m4s(&video.video_path)
        .map_err(|e| ConvertError::Mp4BoxFailed(e.to_string()))?;
    let (audio_clean, audio_temp) = ensure_clean_m4s(&video.audio_path)
        .map_err(|e| ConvertError::Mp4BoxFailed(e.to_string()))?;
    let _cleanup = TempCleanup::new(video_temp.then_some(video_clean.clone()), audio_temp.then_some(audio_clean.clone()));

    let video_str = video_clean.to_string_lossy();
    let audio_str = audio_clean.to_string_lossy();
    let out_str = output_path.to_string_lossy();

    let args = [
        "-add",
        &format!("{}#video", video_str),
        "-add",
        &format!("{}#audio", audio_str),
        "-new",
        &out_str,
        "-itags",
        "tool=Bili2MP4",
    ];

    let output = Command::new(mp4box_path)
        .args(&args)
        .output()
        .map_err(|_| ConvertError::Mp4BoxNotFound)?;

    if cancel.load(Ordering::Relaxed) {
        let _ = std::fs::remove_file(&output_path);
        return Err(ConvertError::Cancelled);
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut err_msg = stderr.to_string();
        if err_msg.len() > 200 {
            err_msg.truncate(200);
            err_msg.push_str("...");
        }
        return Err(ConvertError::Mp4BoxFailed(err_msg));
    }

    on_progress(ConvertProgress {
        current_file: video.title.clone(),
        current_index: 1,
        total: 1,
        percent: 100,
    });

    Ok(output_path)
}

/// 若 MP4Box 均失败，尝试 ffmpeg 合并（部分 B 站 m4s 格式兼容性更好）
pub fn convert_one_ffmpeg(
    video: &VideoInfo,
    out_dir: &Path,
    ffmpeg_path: &str,
    strategy: ConflictStrategy,
    cancel: &AtomicBool,
) -> Result<std::path::PathBuf, ConvertError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(ConvertError::Cancelled);
    }
    if !video.video_path.exists() || !video.audio_path.exists() {
        return Err(ConvertError::FileNotFound("video or audio".to_string()));
    }

    let output_path = resolve_output_path(out_dir, &video.title, strategy)?;

    let (video_clean, video_temp) = ensure_clean_m4s(&video.video_path)
        .map_err(|e| ConvertError::Mp4BoxFailed(e.to_string()))?;
    let (audio_clean, audio_temp) = ensure_clean_m4s(&video.audio_path)
        .map_err(|e| ConvertError::Mp4BoxFailed(e.to_string()))?;
    let _cleanup = TempCleanup::new(video_temp.then_some(video_clean.clone()), audio_temp.then_some(audio_clean.clone()));

    let video_str = video_clean.to_string_lossy();
    let audio_str = audio_clean.to_string_lossy();
    let out_str = output_path.to_string_lossy();

    // ffmpeg -y -i video.m4s -i audio.m4s -c copy -movflags +faststart output.mp4
    let output = Command::new(ffmpeg_path)
        .args(["-y", "-i", &video_str, "-i", &audio_str, "-c", "copy", "-movflags", "+faststart", &out_str])
        .output()
        .map_err(|_| ConvertError::Mp4BoxFailed("ffmpeg 未找到，请安装: brew install ffmpeg".to_string()))?;

    if cancel.load(Ordering::Relaxed) {
        let _ = std::fs::remove_file(&output_path);
        return Err(ConvertError::Cancelled);
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut err_msg = stderr.to_string();
        if err_msg.len() > 200 {
            err_msg.truncate(200);
            err_msg.push_str("...");
        }
        return Err(ConvertError::Mp4BoxFailed(format!("ffmpeg: {}", err_msg)));
    }

    Ok(output_path)
}

/// 若标准 #video/#audio 失败，可调用此函数尝试 :raw 模式
pub fn convert_one_raw(
    video: &VideoInfo,
    out_dir: &Path,
    mp4box_path: &str,
    strategy: ConflictStrategy,
    on_progress: impl Fn(ConvertProgress),
    cancel: &AtomicBool,
) -> Result<std::path::PathBuf, ConvertError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(ConvertError::Cancelled);
    }

    if !video.video_path.exists() || !video.audio_path.exists() {
        return Err(ConvertError::FileNotFound("video or audio".to_string()));
    }

    let output_path = resolve_output_path(out_dir, &video.title, strategy)?;

    on_progress(ConvertProgress {
        current_file: video.title.clone(),
        current_index: 1,
        total: 1,
        percent: 0,
    });

    let (video_clean, video_temp) = ensure_clean_m4s(&video.video_path)
        .map_err(|e| ConvertError::Mp4BoxFailed(e.to_string()))?;
    let (audio_clean, audio_temp) = ensure_clean_m4s(&video.audio_path)
        .map_err(|e| ConvertError::Mp4BoxFailed(e.to_string()))?;
    let _cleanup = TempCleanup::new(video_temp.then_some(video_clean.clone()), audio_temp.then_some(audio_clean.clone()));

    let video_str = video_clean.to_string_lossy();
    let audio_str = audio_clean.to_string_lossy();
    let out_str = output_path.to_string_lossy();

    let args = [
        "-add",
        &format!("{}#video:raw", video_str),
        "-add",
        &format!("{}#audio:raw", audio_str),
        "-new",
        &out_str,
        "-itags",
        "tool=Bili2MP4",
    ];

    let output = Command::new(mp4box_path)
        .args(&args)
        .output()
        .map_err(|_| ConvertError::Mp4BoxNotFound)?;

    if cancel.load(Ordering::Relaxed) {
        let _ = std::fs::remove_file(&output_path);
        return Err(ConvertError::Cancelled);
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ConvertError::Mp4BoxFailed(stderr.to_string()));
    }

    on_progress(ConvertProgress {
        current_file: video.title.clone(),
        current_index: 1,
        total: 1,
        percent: 100,
    });

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_ensure_clean_m4s_no_header() {
        let tmp = std::env::temp_dir().join("bili2mp4_m4s_test");
        fs::create_dir_all(&tmp).ok();
        let p = tmp.join("video.m4s");
        fs::write(&p, b"not_0x30_padding").ok();
        let (out, cleaned) = ensure_clean_m4s(&p).unwrap();
        assert!(!cleaned);
        assert_eq!(out, p);
    }

    #[test]
    fn test_ensure_clean_m4s_with_padding() {
        let tmp = std::env::temp_dir().join("bili2mp4_m4s_test2");
        fs::create_dir_all(&tmp).ok();
        let p = tmp.join("video2.m4s");
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(&[0x30u8; 9]).unwrap();
        f.write_all(b"rest_of_file").unwrap();
        drop(f);
        let (out, cleaned) = ensure_clean_m4s(&p).unwrap();
        assert!(cleaned);
        assert_ne!(out, p);
        let content = fs::read(&out).unwrap();
        assert_eq!(content, b"rest_of_file");
    }
}
