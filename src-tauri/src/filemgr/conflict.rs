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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_overwrite() {
        let out = std::env::temp_dir().join("bili2mp4_test_overwrite");
        std::fs::create_dir_all(&out).ok();
        let p = resolve_output_path(&out, "test", ConflictStrategy::Overwrite).unwrap();
        assert!(p.ends_with("test.mp4"));
    }

    #[test]
    fn test_resolve_rename() {
        let out = std::env::temp_dir().join("bili2mp4_test_rename");
        std::fs::create_dir_all(&out).ok();
        let existing = out.join("video.mp4");
        std::fs::write(&existing, b"").ok();
        let p = resolve_output_path(&out, "video", ConflictStrategy::Rename).unwrap();
        assert!(p.ends_with("video(1).mp4"));
        std::fs::remove_file(existing).ok();
    }

    #[test]
    fn test_resolve_skip() {
        let out = std::env::temp_dir().join("bili2mp4_test_skip");
        std::fs::create_dir_all(&out).ok();
        let existing = out.join("exists.mp4");
        std::fs::write(&existing, b"").ok();
        let r = resolve_output_path(&out, "exists", ConflictStrategy::Skip);
        assert!(r.is_err());
        std::fs::remove_file(existing).ok();
    }
}
