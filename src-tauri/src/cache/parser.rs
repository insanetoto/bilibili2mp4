//! 解析 B 站缓存 entry.json
//! 支持多种 entry.json 结构变体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    /// 缓存目录绝对路径（entry.json 所在目录）
    #[serde(serialize_with = "path_to_string", deserialize_with = "string_to_path")]
    pub cache_dir: PathBuf,
    /// 视频标题
    pub title: String,
    /// 清晰度描述，如 1080P、720P
    pub quality: String,
    /// 分 P 序号，从 1 开始
    pub page: u32,
    /// 总 P 数
    pub total_pages: u32,
    /// 视频大小（字节）
    pub size_bytes: u64,
    /// 缓存时间
    pub cached_at: Option<String>,
    /// video.m4s 绝对路径
    #[serde(serialize_with = "path_to_string", deserialize_with = "string_to_path")]
    pub video_path: PathBuf,
    /// audio.m4s 绝对路径
    #[serde(serialize_with = "path_to_string", deserialize_with = "string_to_path")]
    pub audio_path: PathBuf,
}

fn path_to_string<S>(path: &PathBuf, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&path.display().to_string())
}

fn string_to_path<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let st = String::deserialize(d)?;
    Ok(PathBuf::from(st))
}

/// 解析 videoInfo.json（新版 B 站 macOS 客户端）
pub fn parse_video_info(info_path: &Path) -> Result<VideoInfo, ParseError> {
    let content = std::fs::read_to_string(info_path)?;
    let info: VideoInfoJson = serde_json::from_str(&content)?;

    let cache_dir = info_path
        .parent()
        .ok_or(ParseError::MissingMedia)?
        .to_path_buf();

    let (video_path, audio_path) = find_m4s_files_by_id(&cache_dir, info.item_id)?;

    let title = info
        .tab_name
        .or(info.title)
        .unwrap_or_else(|| "未知标题".to_string());

    let quality = _qn_to_quality(info.qn).unwrap_or_else(|| "未知".to_string());

    let page = info.p.unwrap_or(1);
    let total_pages = 1.max(page);

    let size_bytes = std::fs::metadata(&video_path).map(|m| m.len()).unwrap_or(0)
        + std::fs::metadata(&audio_path).map(|m| m.len()).unwrap_or(0);

    let cached_at = std::fs::metadata(info_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let dt: DateTime<Utc> = t.into();
            dt.format("%Y-%m-%d").to_string()
        });

    Ok(VideoInfo {
        cache_dir,
        title,
        quality,
        page,
        total_pages,
        size_bytes,
        cached_at,
        video_path,
        audio_path,
    })
}

#[derive(Debug, Deserialize)]
struct VideoInfoJson {
    #[serde(default, alias = "itemId")]
    item_id: Option<u64>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default, alias = "tabName")]
    tab_name: Option<String>,
    #[serde(default)]
    p: Option<u32>,
    #[serde(default)]
    qn: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct EntryJson {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    page_data: Option<PageData>,
    #[serde(default)]
    type_tag: Option<String>,
    #[serde(default)]
    quality: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    bvid: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    avid: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    tid: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct PageData {
    #[serde(default)]
    page: Option<u32>,
    #[serde(default)]
    part: Option<String>,
    #[serde(default)]
    index_title: Option<String>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("无法读取 entry.json: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("缺少视频或音频文件")]
    MissingMedia,
}

/// 解析 entry.json，结合目录内 m4s 文件返回 VideoInfo
pub fn parse_entry(entry_path: &Path) -> Result<VideoInfo, ParseError> {
    let content = std::fs::read_to_string(entry_path)?;
    let entry: EntryJson = serde_json::from_str(&content)?;

    let cache_dir = entry_path
        .parent()
        .ok_or(ParseError::MissingMedia)?
        .to_path_buf();

    let (video_path, audio_path) = find_m4s_files(&cache_dir)?;

    let title = entry
        .page_data
        .as_ref()
        .and_then(|p| p.part.clone())
        .or(entry.page_data.as_ref().and_then(|p| p.index_title.clone()))
        .or(entry.title.clone())
        .unwrap_or_else(|| "未知标题".to_string());

    let quality = _type_tag_to_quality(entry.type_tag.as_deref())
        .or_else(|| _quality_to_str(entry.quality))
        .unwrap_or_else(|| "未知".to_string());

    let page = entry.page_data.as_ref().and_then(|p| p.page).unwrap_or(1);
    let total_pages = 1.max(page);

    let size_bytes = std::fs::metadata(&video_path).map(|m| m.len()).unwrap_or(0)
        + std::fs::metadata(&audio_path).map(|m| m.len()).unwrap_or(0);

    let cached_at = std::fs::metadata(entry_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let dt: DateTime<Utc> = t.into();
            dt.format("%Y-%m-%d").to_string()
        });

    Ok(VideoInfo {
        cache_dir,
        title,
        quality,
        page,
        total_pages,
        size_bytes,
        cached_at,
        video_path,
        audio_path,
    })
}

/// 按 item_id 查找 m4s 文件（格式：{item_id}-1-{codec}.m4s）
/// 约定：较大 codec 为视频，较小为音频
fn find_m4s_files_by_id(cache_dir: &Path, item_id: Option<u64>) -> Result<(PathBuf, PathBuf), ParseError> {
    let item_id = item_id.or_else(|| {
        cache_dir
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|s| s.parse().ok())
    }).ok_or(ParseError::MissingMedia)?;

    let prefix = format!("{}-1-", item_id);
    let mut m4s_files: Vec<_> = std::fs::read_dir(cache_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "m4s")
                .unwrap_or(false)
        })
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.starts_with(&prefix))
                .unwrap_or(false)
        })
        .collect();

    // 按文件名尾部的 codec 数字降序排序，较大者通常为视频（质量码），较小为音频
    let extract_code = |p: &std::fs::DirEntry| {
        p.path()
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.rsplit('-').next())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
    };
    m4s_files.sort_by(|a, b| extract_code(b).cmp(&extract_code(a)));

    if m4s_files.len() < 2 {
        return Err(ParseError::MissingMedia);
    }

    // 较大的 codec 为视频，较小的为音频
    let (video_idx, audio_idx) = (0, 1);

    Ok((
        m4s_files[video_idx].path(),
        m4s_files[audio_idx].path(),
    ))
}

/// 在目录及子目录中查找 video.m4s 和 audio.m4s
fn find_m4s_files(cache_dir: &Path) -> Result<(PathBuf, PathBuf), ParseError> {
    let mut video_path = None;
    let mut audio_path = None;

    let subdirs = ["64", "80", "32", "16", "112", "116", "120", ""];
    for sub in subdirs {
        let base = if sub.is_empty() {
            cache_dir.to_path_buf()
        } else {
            cache_dir.join(sub)
        };
        if !base.exists() {
            continue;
        }
        let v = base.join("video.m4s");
        let a = base.join("audio.m4s");
        if v.exists() && a.exists() {
            video_path = Some(v);
            audio_path = Some(a);
            break;
        }
    }

    if video_path.is_none() || audio_path.is_none() {
        return Err(ParseError::MissingMedia);
    }

    Ok((video_path.unwrap(), audio_path.unwrap()))
}

#[cfg(test)]
pub(crate) fn type_tag_to_quality(tag: Option<&str>) -> Option<String> {
    _type_tag_to_quality(tag)
}

fn _type_tag_to_quality(tag: Option<&str>) -> Option<String> {
    let s = tag?;
    Some(match s {
        "s_1080p" | "1080" => "1080P".to_string(),
        "s_720p" | "720" => "720P".to_string(),
        "s_480p" | "480" => "480P".to_string(),
        "s_360p" | "360" => "360P".to_string(),
        "s_240p" | "240" => "240P".to_string(),
        _ if s.contains("1080") => "1080P".to_string(),
        _ if s.contains("720") => "720P".to_string(),
        _ if s.contains("480") => "480P".to_string(),
        _ => s.to_string(),
    })
}

fn _quality_to_str(q: Option<u32>) -> Option<String> {
    q.map(|n| format!("{}P", n))
}

fn _qn_to_quality(qn: Option<u32>) -> Option<String> {
    qn.map(|n| match n {
        120 => "1080P+".to_string(),
        116 => "1080P60".to_string(),
        112 => "1080P".to_string(),
        80 => "720P60".to_string(),
        74 => "720P".to_string(),
        64 => "480P".to_string(),
        32 => "360P".to_string(),
        16 => "240P".to_string(),
        _ => format!("{}P", n),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_type_tag_to_quality() {
        assert_eq!(type_tag_to_quality(Some("s_1080p")), Some("1080P".into()));
        assert_eq!(type_tag_to_quality(Some("s_720p")), Some("720P".into()));
        assert_eq!(type_tag_to_quality(Some("xxx")), Some("xxx".into()));
        assert_eq!(type_tag_to_quality(None), None);
    }

    #[test]
    fn test_parse_entry_with_mock_files() {
        let tmp = std::env::temp_dir().join("bili2mp4_parse_test");
        fs::create_dir_all(&tmp).ok();
        let sub = tmp.join("80");
        fs::create_dir_all(&sub).ok();
        fs::write(sub.join("video.m4s"), b"x").ok();
        fs::write(sub.join("audio.m4s"), b"x").ok();
        let entry = r#"{"title":"测试","page_data":{"page":1,"part":"Part1"},"type_tag":"s_1080p"}"#;
        fs::write(tmp.join("entry.json"), entry).ok();
        let r = parse_entry(&tmp.join("entry.json"));
        assert!(r.is_ok(), "{:?}", r.err());
        let v = r.unwrap();
        assert_eq!(v.title, "Part1");
        assert_eq!(v.quality, "1080P");
        fs::remove_dir_all(&tmp).ok();
    }
}
