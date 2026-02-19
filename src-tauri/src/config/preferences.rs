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

/// 检测 MP4Box 路径：优先配置，其次 which，最后返回默认 "MP4Box"
pub fn resolve_mp4box_path(config: &AppConfig) -> String {
    if let Some(ref p) = config.mp4box_path {
        if std::path::Path::new(p).exists() {
            return p.clone();
        }
    }
    if let Ok(path) = which_mp4box() {
        return path;
    }
    "MP4Box".to_string()
}

fn which_mp4box() -> Result<String, std::io::Error> {
    let output = std::process::Command::new("which")
        .arg("MP4Box")
        .output()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);
        let s = s.trim();
        if !s.is_empty() {
            return Ok(s.to_string());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "MP4Box not found",
    ))
}
