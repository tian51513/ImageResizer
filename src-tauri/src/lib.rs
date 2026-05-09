mod commands;
mod config;
mod processor;
mod scanner;
mod state;

use simplelog::{Config, LevelFilter, WriteLogger};
use state::AppState;
use std::fs::{self, OpenOptions};
use std::sync::Mutex;

fn init_logging() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let log_dir = exe_dir.join("logs");
    let _ = fs::create_dir_all(&log_dir);

    let log_path = log_dir.join(format!(
        "{}.log",
        chrono::Local::now().format("%Y-%m-%d")
    ));
    match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(f) => {
            let _ = WriteLogger::init(LevelFilter::Info, Config::default(), f);
            log::info!("Logging initialized at {}", log_path.display());
        }
        Err(e) => {
            eprintln!("Failed to create log file {}: {}", log_path.display(), e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![
            commands::scan_directory,
            commands::get_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::start_processing,
            commands::stop_processing,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
