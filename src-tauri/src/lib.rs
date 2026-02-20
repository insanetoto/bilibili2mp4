//! Bili2MP4 Tauri 应用

pub mod cache;
pub mod config;
pub mod convert;
pub mod filemgr;

use cache::{scan, VideoInfo};
use config::{load_config, resolve_ffmpeg_path, resolve_mp4box_path, save_config, AppConfig};
use convert::{convert_one, convert_one_raw, convert_one_ffmpeg, ConvertError, ConvertProgress};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

static CONVERT_CANCEL: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

#[tauri::command]
fn scan_cache(dir: Option<String>) -> Result<Vec<VideoInfo>, String> {
    let dir_path = match dir {
        Some(d) => PathBuf::from(d),
        None => {
            let paths = cache::scanner::default_cache_paths();
            paths
                .into_iter()
                .find(|p| p.exists())
                .ok_or_else(|| "未找到默认缓存目录，请手动选择".to_string())?
        }
    };
    scan(&dir_path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn convert(
    app: tauri::AppHandle,
    items: Vec<VideoInfo>,
    out_dir: String,
) -> Result<Vec<String>, String> {
    let config = load_config();
    let mp4box = resolve_mp4box_path(&config);
    let ffmpeg = resolve_ffmpeg_path();
    let strategy = config.conflict_strategy();
    let out_path = PathBuf::from(&out_dir);

    if !out_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&out_path) {
            return Err(format!("无法创建输出目录: {}", e));
        }
    }

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut guard = CONVERT_CANCEL.lock().unwrap();
        *guard = Some(Arc::clone(&cancel));
    }

    let mp4box_note = if mp4box == "MP4Box" || mp4box == "MP4Box.exe" {
        #[cfg(target_os = "macos")]
        { " (未找到，请 brew install gpac)" }
        #[cfg(target_os = "windows")]
        { " (未找到，请安装 GPAC 或配置路径)" }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        { " (未找到)" }
    } else {
        ""
    };
    let _ = app.emit("convert-log", serde_json::json!({
        "level": if mp4box == "MP4Box" || mp4box == "MP4Box.exe" { "warn" } else { "info" },
        "message": format!("MP4Box 路径: {}{}", mp4box, mp4box_note)
    }));
    let ffmpeg_note = if ffmpeg == "ffmpeg" || ffmpeg == "ffmpeg.exe" {
        #[cfg(target_os = "macos")]
        { " (未找到，兜底时需 brew install ffmpeg)" }
        #[cfg(target_os = "windows")]
        { " (未找到，兜底时需安装 ffmpeg)" }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        { " (未找到)" }
    } else {
        ""
    };
    let _ = app.emit("convert-log", serde_json::json!({
        "level": if ffmpeg == "ffmpeg" || ffmpeg == "ffmpeg.exe" { "warn" } else { "info" },
        "message": format!("ffmpeg 路径: {}{}", ffmpeg, ffmpeg_note)
    }));
    let _ = app.emit("convert-log", serde_json::json!({ "level": "info", "message": format!("输出目录: {}", out_path.display()) }));
    let _ = app.emit("convert-log", serde_json::json!({ "level": "info", "message": "--- 开始转换 ---" }));

    let app_clone = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut success_paths = Vec::new();
        let total = items.len();

        for (i, video) in items.into_iter().enumerate() {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "warn", "message": "用户取消" }));
                break;
            }

            let progress = |p: ConvertProgress| {
                let payload = ConvertProgress {
                    current_index: i + 1,
                    total,
                    percent: (100 * (i + 1) / total.max(1)) as u32,
                    ..p
                };
                let _ = app_clone.emit("convert-progress", &payload);
            };

            let _ = app_clone.emit("convert-log", serde_json::json!({
                "level": "info",
                "message": format!("[{}/{}] 正在转换: {}", i + 1, total, video.title)
            }));

            match convert_one(&video, &out_path, &mp4box, strategy, &progress, &*cancel) {
                Ok(path) => {
                    let s = path.display().to_string();
                    let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "info", "message": format!("  ✓ 成功: {}", s) }));
                    success_paths.push(s);
                }
                Err(ConvertError::Mp4BoxFailed(e)) => {
                    let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "warn", "message": format!("  MP4Box 失败: {}", e) }));
                    let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "info", "message": "  尝试 :raw 模式..." }));
                    let ok = convert_one_raw(
                        &video,
                        &out_path,
                        &mp4box,
                        strategy,
                        &progress,
                        &*cancel,
                    );
                    match &ok {
                        Ok(p) => {
                            let s = p.display().to_string();
                            let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "info", "message": format!("  ✓ raw 成功: {}", s) }));
                            success_paths.push(s);
                        }
                        Err(e) => {
                            let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "warn", "message": format!("  raw 失败: {}", e) }));
                            let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "info", "message": "  尝试 ffmpeg 兜底..." }));
                            if let Ok(p) = convert_one_ffmpeg(&video, &out_path, &ffmpeg, strategy, &*cancel) {
                                let s = p.display().to_string();
                                let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "info", "message": format!("  ✓ ffmpeg 成功: {}", s) }));
                                success_paths.push(s);
                            } else {
                                let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "error", "message": format!("  ✗ ffmpeg 也失败") }));
                            }
                        }
                    }
                }
                Err(ConvertError::Skipped(_)) => {
                    let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "info", "message": "  跳过 (输出文件已存在)" }));
                }
                Err(ConvertError::Cancelled) => break,
                Err(e) => {
                    let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "error", "message": format!("  ✗ 失败: {}", e) }));
                }
            }
        }

        let _ = app_clone.emit("convert-log", serde_json::json!({ "level": "info", "message": format!("--- 完成，成功 {} 个 ---", success_paths.len()) }));
        success_paths
    })
    .await
    .map_err(|e| format!("转换任务异常: {}", e))?;

    {
        let mut guard = CONVERT_CANCEL.lock().unwrap();
        *guard = None;
    }

    Ok(result)
}

#[tauri::command]
fn cancel_convert() {
    if let Ok(guard) = CONVERT_CANCEL.lock() {
        if let Some(ref c) = *guard {
            c.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

#[tauri::command]
fn get_config() -> AppConfig {
    load_config()
}

#[tauri::command]
fn set_config(config: AppConfig) -> Result<(), String> {
    save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
fn default_cache_paths() -> Vec<String> {
    cache::scanner::default_cache_paths()
        .into_iter()
        .filter_map(|p| p.to_str().map(String::from))
        .collect()
}

#[tauri::command]
fn default_output_dir() -> Option<String> {
    dirs::download_dir()
        .and_then(|p| p.to_str().map(String::from))
}

/// 测试模式用：报告转换测试结果（TAURI_TEST_CONVERT=1 时由前端调用）
#[tauri::command]
fn report_test_result(_success: bool, _message: String) {}

/// 在资源管理器中打开文件夹（macOS: open，Windows: explorer）
#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    let status = {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(&path).spawn()
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer").arg(&path).spawn()
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = path;
            return Err("当前平台暂不支持打开文件夹".to_string());
        }
    };
    status.map_err(|e| format!("无法打开文件夹: {}", e))?;
    Ok(())
}

/// 选择缓存目录对话框的默认路径
#[tauri::command]
fn default_cache_dialog_path() -> Option<String> {
    let home = dirs::home_dir()?;
    #[cfg(target_os = "macos")]
    {
        let bilibili = home.join("Movies/bilibili");
        if bilibili.exists() {
            return bilibili.to_str().map(String::from);
        }
        let alt = home.join("Movies/Bilibili");
        if alt.exists() {
            return alt.to_str().map(String::from);
        }
    }
    #[cfg(target_os = "windows")]
    {
        let local = dirs::data_local_dir()?;
        let bilibili = local.join("bilibili").join("download");
        if bilibili.exists() {
            return bilibili.to_str().map(String::from);
        }
        if let Some(p) = std::fs::read_dir(local.join("Packages")).ok().and_then(|d| {
            d.filter_map(|e| e.ok())
                .find(|e| e.file_name().to_str().map(|s| s.starts_with("Microsoft.48666Bilibili")).unwrap_or(false))
        }) {
            let sub = p.path().join("LocalState").join("download");
            if sub.exists() {
                return sub.to_str().map(String::from);
            }
        }
    }
    dirs::document_dir().and_then(|p| p.to_str().map(String::from))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(_test_convert: bool) {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            scan_cache,
            convert,
            cancel_convert,
            get_config,
            set_config,
            default_cache_paths,
            default_output_dir,
            default_cache_dialog_path,
            open_folder,
            report_test_result,
        ])
        .setup(move |app| {
            let test = std::env::args().any(|a| a == "--test-convert")
                || std::env::var("TAURI_TEST_CONVERT").as_deref() == Ok("1");
            if test {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(4));
                    let _ = handle.emit("run-test-convert", ());
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}
