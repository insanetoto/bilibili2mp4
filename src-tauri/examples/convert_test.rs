//! 测试转换：cargo run --example convert_test

use bili2mp4::cache::{scan, VideoInfo};
use bili2mp4::config::{load_config, resolve_mp4box_path};
use bili2mp4::convert::{convert_one, ConvertError};
use bili2mp4::filemgr::ConflictStrategy;
use std::path::Path;
use std::sync::atomic::AtomicBool;

fn main() {
    let cache_dir = "/Users/xinz/Movies/bilibili/1318051900";
    let out_dir = "/Users/xinz/Downloads";

    println!("扫描: {}", cache_dir);
    let videos = scan(Path::new(cache_dir)).expect("扫描失败");
    println!("找到 {} 个视频", videos.len());

    if videos.is_empty() {
        println!("无视频可转换");
        return;
    }

    let config = load_config();
    let mp4box = resolve_mp4box_path(&config);
    println!("MP4Box: {}", mp4box);

    for (i, v) in videos.iter().enumerate() {
        println!("\n[{}/{}] {}", i + 1, videos.len(), v.title);
        println!("  视频: {:?}", v.video_path);
        println!("  音频: {:?}", v.audio_path);
    }

    let cancel = AtomicBool::new(false);
    let video = &videos[0];

    println!("\n开始转换到: {}", out_dir);
    match convert_one(
        video,
        Path::new(out_dir),
        &mp4box,
        ConflictStrategy::Rename,
        |p| println!("  进度: {} {}%", p.current_file, p.percent),
        &cancel,
    ) {
        Ok(path) => println!("\n✓ 完成: {}", path.display()),
        Err(ConvertError::Mp4BoxNotFound) => {
            println!("\n✗ MP4Box 未找到，请运行: brew install gpac");
        }
        Err(e) => println!("\n✗ 失败: {}", e),
    }
}
