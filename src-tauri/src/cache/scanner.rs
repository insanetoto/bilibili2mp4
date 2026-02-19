//! 扫描 B 站缓存目录，定位 entry.json 并解析

use super::parser::{parse_entry, parse_video_info, VideoInfo};
use std::path::PathBuf;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("目录不存在: {0}")]
    DirNotFound(PathBuf),
    #[error("解析失败: {0}")]
    Parse(#[from] super::parser::ParseError),
}

/// 默认 B 站 macOS 缓存路径
pub fn default_cache_paths() -> Vec<PathBuf> {
    dirs::home_dir()
        .map(|home| {
            vec![
                home.join("Movies/bilibili"),
                home.join("Movies/Bilibili"),
                home.join("Library/Containers/com.bilibili.bilibili/Data/Download"),
            ]
        })
        .unwrap_or_default()
}

/// 扫描指定目录，返回所有可解析的视频
pub fn scan(dir: &std::path::Path) -> Result<Vec<VideoInfo>, ScanError> {
    if !dir.exists() {
        return Err(ScanError::DirNotFound(dir.to_path_buf()));
    }

    let mut videos = Vec::new();
    let mut seen_dirs = std::collections::HashSet::new();

    for entry in WalkDir::new(dir)
        .max_depth(6)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        let is_entry = p.file_name().map(|n| n == "entry.json").unwrap_or(false);
        let is_video_info = p.file_name().map(|n| n == "videoInfo.json" || n == ".videoInfo").unwrap_or(false);

        if is_entry {
            if let Some(parent) = p.parent() {
                if seen_dirs.insert(parent.to_path_buf()) {
                    if let Ok(v) = parse_entry(p) {
                        videos.push(v);
                    }
                }
            }
        } else if is_video_info {
            if let Some(parent) = p.parent() {
                if seen_dirs.insert(parent.to_path_buf()) {
                    if let Ok(v) = parse_video_info(p) {
                        videos.push(v);
                    }
                }
            }
        }
    }

    Ok(videos)
}
