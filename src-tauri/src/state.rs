use crate::config::ConfigManager;
use std::sync::atomic::{AtomicBool, Ordering};
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

/// RAII guard that keeps the is_processing flag raised until dropped —
/// including on panic or early return — so the UI can never get stuck.
pub struct ProcessingGuard {
    flag: Arc<AtomicBool>,
}

impl ProcessingGuard {
    pub fn new(flag: Arc<AtomicBool>) -> Self {
        flag.store(true, Ordering::SeqCst);
        ProcessingGuard { flag }
    }
}

impl Drop for ProcessingGuard {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_raises_and_clears_flag() {
        let flag = Arc::new(AtomicBool::new(false));
        {
            let _g = ProcessingGuard::new(Arc::clone(&flag));
            assert!(flag.load(Ordering::SeqCst), "flag must be raised while held");
        }
        assert!(!flag.load(Ordering::SeqCst), "flag must clear on drop");
    }

    #[test]
    fn guard_clears_flag_even_on_panic() {
        let flag = Arc::new(AtomicBool::new(false));
        let inner = Arc::clone(&flag);
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {})); // silence expected panic
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _g = ProcessingGuard::new(inner);
            panic!("simulated worker panic");
        }));
        std::panic::set_hook(prev_hook);
        assert!(
            !flag.load(Ordering::SeqCst),
            "flag must clear even when the holder panics"
        );
    }
}
