mod commands;
mod config;
mod processor;
mod scanner;
mod state;

use state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
