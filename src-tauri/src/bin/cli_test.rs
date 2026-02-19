//! 命令行测试：扫描 + 转换
//! 运行: cargo run --bin cli_test [缓存目录] [输出目录]

fn main() {
    let cache_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join("Movies/bilibili").display().to_string())
                .unwrap_or_else(|| "/tmp".to_string())
        });
    let out_dir = std::env::args()
        .nth(2)
        .unwrap_or_else(|| dirs::download_dir().map(|p| p.display().to_string()).unwrap_or_else(|| "/tmp".to_string()));

    println!("=== Bili2MP4 CLI 测试 ===");
    println!("缓存目录: {}", cache_dir);
    println!("输出目录: {}", out_dir);

    let path = std::path::Path::new(&cache_dir);
    if !path.exists() {
        eprintln!("错误: 缓存目录不存在");
        std::process::exit(1);
    }

    println!("\n[1] 扫描缓存...");
    let videos = match bili2mp4::cache::scan(path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("扫描失败: {}", e);
            std::process::exit(1);
        }
    };

    println!("找到 {} 个视频", videos.len());
    if videos.is_empty() {
        std::process::exit(0);
    }

    for (i, v) in videos.iter().take(3).enumerate() {
        println!("  {}: {} ({})", i + 1, v.title, v.quality);
    }
    if videos.len() > 3 {
        println!("  ...");
    }

    let config = bili2mp4::config::load_config();
    let mp4box = bili2mp4::config::resolve_mp4box_path(&config);
    println!("\n[2] MP4Box 路径: {}", mp4box);

    let out_path = std::path::Path::new(&out_dir);
    std::fs::create_dir_all(out_path).expect("创建输出目录失败");

    let cancel = std::sync::atomic::AtomicBool::new(false);
    let strategy = config.conflict_strategy();

    println!("\n[3] 转换第一个视频...");
    let video = &videos[0];
    let result = bili2mp4::convert::convert_one(
        video,
        out_path,
        &mp4box,
        strategy,
        |p| println!("  进度: {}%", p.percent),
        &cancel,
    );

    match result {
        Ok(p) => println!("成功: {}", p.display()),
        Err(e) => {
            eprintln!("失败: {:?}", e);
            std::process::exit(1);
        }
    }
}
