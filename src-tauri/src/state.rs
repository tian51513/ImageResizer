use crate::config::ConfigManager;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct AppState {
    pub config_manager: ConfigManager,
    pub stop_flag: Arc<AtomicBool>,
    pub is_processing: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            config_manager: ConfigManager::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            is_processing: Arc::new(AtomicBool::new(false)),
        }
    }
}
