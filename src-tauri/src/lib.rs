//! Bili2MP4 Tauri 应用

pub mod cache;
pub mod config;
pub mod convert;
pub mod filemgr;

use cache::{scan, VideoInfo};
use config::{load_config, resolve_mp4box_path, save_config, AppConfig};
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

    let app_clone = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut success_paths = Vec::new();
        let total = items.len();

        for (i, video) in items.into_iter().enumerate() {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
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

            match convert_one(&video, &out_path, &mp4box, strategy, &progress, &*cancel) {
                Ok(path) => success_paths.push(path.display().to_string()),
                Err(ConvertError::Mp4BoxFailed(_)) => {
                    let ok = convert_one_raw(
                        &video,
                        &out_path,
                        &mp4box,
                        strategy,
                        &progress,
                        &*cancel,
                    )
                    .or_else(|_| convert_one_ffmpeg(&video, &out_path, strategy, &*cancel));
                    if let Ok(p) = ok {
                        success_paths.push(p.display().to_string());
                    }
                }
                Err(ConvertError::Skipped(_)) => {}
                Err(ConvertError::Cancelled) => break,
                Err(_) => {}
            }
        }

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

/// 选择缓存目录对话框的默认路径：~/Movies/bilibili 存在则用，否则 ~/Documents
#[tauri::command]
fn default_cache_dialog_path() -> Option<String> {
    let home = dirs::home_dir()?;
    let bilibili = home.join("Movies/bilibili");
    if bilibili.exists() {
        return bilibili.to_str().map(String::from);
    }
    let alt = home.join("Movies/Bilibili");
    if alt.exists() {
        return alt.to_str().map(String::from);
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
