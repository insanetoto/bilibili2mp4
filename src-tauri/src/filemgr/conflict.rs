//! 文件冲突处理

use super::output;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum ConflictStrategy {
    /// 覆盖已存在文件
    Overwrite,
    /// 跳过，不转换
    Skip,
    /// 自动重命名，如 标题(1).mp4
    #[default]
    Rename,
}

#[derive(Debug, Error)]
pub enum ConflictError {
    #[error("用户选择跳过")]
    Skip,
}

/// 根据策略解析最终输出路径
pub fn resolve_output_path(
    out_dir: &Path,
    title: &str,
    strategy: ConflictStrategy,
) -> Result<PathBuf, ConflictError> {
    let path = output::output_path(out_dir, title);

    match strategy {
        ConflictStrategy::Overwrite => Ok(path),
        ConflictStrategy::Skip => {
            if path.exists() {
                Err(ConflictError::Skip)
            } else {
                Ok(path)
            }
        }
        ConflictStrategy::Rename => {
            if !path.exists() {
                return Ok(path);
            }
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("video").to_string();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("mp4").to_string();
            let mut n = 1;
            loop {
                let name = format!("{}({}).{}", stem, n, ext);
                let next = out_dir.join(&name);
                if !next.exists() {
                    break Ok(next);
                }
                n += 1;
            }
        }
    }
}
