//! 偏好配置持久化

use crate::filemgr::ConflictStrategy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    /// 默认输出目录
    pub output_dir: Option<String>,
    /// 完成后操作：none | open_folder | open_and_play | notify
    pub on_complete: Option<String>,
    /// 冲突策略
    pub conflict_strategy: Option<String>,
    /// MP4Box 自定义路径（若未捆绑）
    pub mp4box_path: Option<String>,
}

impl AppConfig {
    pub fn conflict_strategy(&self) -> ConflictStrategy {
        match self.conflict_strategy.as_deref() {
            Some("overwrite") => ConflictStrategy::Overwrite,
            Some("skip") => ConflictStrategy::Skip,
            _ => ConflictStrategy::Rename,
        }
    }
}

/// 配置文件路径：~/.config/bili2mp4/config.json
pub fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("bili2mp4").join("config.json"))
}

pub fn load_config() -> AppConfig {
    let path = match get_config_path() {
        Some(p) => p,
        None => return AppConfig::default(),
    };
    if !path.exists() {
        return AppConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) -> std::io::Result<()> {
    let path = match get_config_path() {
        Some(p) => p,
        None => return Ok(()),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let s = serde_json::to_string_pretty(config)?;
    std::fs::write(path, s)
}

/// Homebrew 常见路径（GUI .app 启动时 PATH 不含这些，需显式查找）
#[cfg(target_os = "macos")]
const BREW_PATHS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin"];

#[cfg(target_os = "macos")]
fn find_in_brew_paths(name: &str) -> Option<String> {
    for base in BREW_PATHS {
        let path = std::path::Path::new(base).join(name);
        if path.exists() {
            if let Some(s) = path.to_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// Windows 常见安装路径
#[cfg(target_os = "windows")]
const WIN_TOOL_PATHS: &[&str] = &[
    r"C:\Program Files\GPAC",
    r"C:\Program Files (x86)\GPAC",
    r"C:\ffmpeg",
    r"C:\Program Files\ffmpeg",
];

#[cfg(target_os = "windows")]
fn find_in_win_paths(mp4box: bool) -> Option<String> {
    let name = if mp4box { "MP4Box.exe" } else { "ffmpeg.exe" };
    for base in WIN_TOOL_PATHS {
        let path = std::path::Path::new(base).join(name);
        if path.exists() {
            return path.to_str().map(String::from);
        }
    }
    None
}

/// 检测 MP4Box 路径：优先配置，其次 which/where，再平台路径，最后默认名称
pub fn resolve_mp4box_path(config: &AppConfig) -> String {
    if let Some(ref p) = config.mp4box_path {
        if std::path::Path::new(p).exists() {
            return p.clone();
        }
    }
    if let Ok(path) = which_mp4box() {
        return path;
    }
    #[cfg(target_os = "macos")]
    return find_in_brew_paths("MP4Box").unwrap_or_else(|| "MP4Box".to_string());
    #[cfg(target_os = "windows")]
    return find_in_win_paths(true).unwrap_or_else(|| "MP4Box.exe".to_string());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    "MP4Box".to_string()
}

/// 检测 ffmpeg 路径（兜底转换用）
pub fn resolve_ffmpeg_path() -> String {
    if let Ok(path) = which_ffmpeg() {
        return path;
    }
    #[cfg(target_os = "macos")]
    return find_in_brew_paths("ffmpeg").unwrap_or_else(|| "ffmpeg".to_string());
    #[cfg(target_os = "windows")]
    return find_in_win_paths(false).unwrap_or_else(|| "ffmpeg.exe".to_string());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    "ffmpeg".to_string()
}

#[cfg(unix)]
fn which_mp4box() -> Result<String, std::io::Error> {
    let output = std::process::Command::new("which").arg("MP4Box").output()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !s.is_empty() {
            return Ok(s);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "MP4Box not found",
    ))
}

#[cfg(windows)]
fn which_mp4box() -> Result<String, std::io::Error> {
    let output = std::process::Command::new("where").arg("MP4Box").output()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").trim().to_string();
        if !s.is_empty() {
            return Ok(s);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "MP4Box not found",
    ))
}

#[cfg(unix)]
fn which_ffmpeg() -> Result<String, std::io::Error> {
    let output = std::process::Command::new("which").arg("ffmpeg").output()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !s.is_empty() {
            return Ok(s);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "ffmpeg not found",
    ))
}

#[cfg(windows)]
fn which_ffmpeg() -> Result<String, std::io::Error> {
    let output = std::process::Command::new("where").arg("ffmpeg").output()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").trim().to_string();
        if !s.is_empty() {
            return Ok(s);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "ffmpeg not found",
    ))
}
