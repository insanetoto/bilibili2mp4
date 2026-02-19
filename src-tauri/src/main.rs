#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let test_convert = std::env::args().any(|a| a == "--test-convert")
        || std::env::var("TAURI_TEST_CONVERT").as_deref() == Ok("1");
    bili2mp4::run(test_convert);
}
