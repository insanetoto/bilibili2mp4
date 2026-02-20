//! 输出路径与文件名安全化

use std::path::PathBuf;

const INVALID_CHARS: [char; 10] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
const MAX_FILENAME_LEN: usize = 200;

/// 过滤非法字符，生成安全的文件名（保留中文、emoji）
pub fn sanitize_filename(name: &str) -> String {
    let s: String = name
        .chars()
        .filter(|c| !INVALID_CHARS.contains(c))
        .collect();
    let s = s.trim();
    let s = if s.is_empty() { "未命名" } else { s };
    if s.chars().count() > MAX_FILENAME_LEN {
        s.chars().take(MAX_FILENAME_LEN).collect()
    } else {
        s.to_string()
    }
}

/// 生成输出文件路径：out_dir / {title}.mp4
pub fn output_path(out_dir: &std::path::Path, title: &str) -> PathBuf {
    let name = sanitize_filename(title);
    out_dir.join(format!("{}.mp4", name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("正常标题"), "正常标题");
        assert_eq!(sanitize_filename("a/b\\c"), "abc");
        assert_eq!(sanitize_filename("  trim  "), "trim");
        assert_eq!(sanitize_filename(""), "未命名");
        assert_eq!(sanitize_filename("   "), "未命名");
    }

    #[test]
    fn test_output_path() {
        let out = Path::new("/tmp");
        assert_eq!(output_path(out, "测试").to_str().unwrap(), "/tmp/测试.mp4");
    }
}
