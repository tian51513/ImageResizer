use crate::batcher::ProgressBatcher;
use crate::config::*;
use crate::processor::ImageProcessor;
use crate::state::{AppState, ProcessingGuard};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<FileMetadata>, String> {
    crate::scanner::scan_directory(&path)
}

#[tauri::command]
pub async fn get_profiles(state: tauri::State<'_, AppState>) -> Result<Vec<Profile>, String> {
    Ok(state.config_manager.load_profiles())
}

#[tauri::command]
pub async fn save_profile(
    state: tauri::State<'_, AppState>,
    profile: Profile,
) -> Result<(), String> {
    let mut profiles = state.config_manager.load_profiles();
    if let Some(pos) = profiles.iter().position(|p| p.name == profile.name) {
        profiles[pos] = profile;
    } else {
        profiles.push(profile);
    }
    state.config_manager.save_profiles(&profiles)
}

#[tauri::command]
pub async fn delete_profile(state: tauri::State<'_, AppState>, name: String) -> Result<(), String> {
    let mut profiles = state.config_manager.load_profiles();
    profiles.retain(|p| p.name != name);
    if profiles.is_empty() {
        return Err("Cannot delete the last profile".to_string());
    }
    state.config_manager.save_profiles(&profiles)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn start_processing(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    files: Vec<FileMetadata>,
    profile: Profile,
    source_dir: String,
) -> Result<(), String> {
    if state.is_processing.load(Ordering::SeqCst) {
        return Err("Processing is already in progress".to_string());
    }
    state.is_processing.store(true, Ordering::SeqCst);
    state.stop_flag.store(false, Ordering::SeqCst);

    let stop_flag = Arc::clone(&state.stop_flag);
    let is_processing = Arc::clone(&state.is_processing);

    tauri::async_runtime::spawn_blocking(move || {
        // RAII guard: the flag clears and the complete event fires no matter
        // how the batch ends — normal return, error, or panic.
        let _guard = ProcessingGuard::new(is_processing);
        let batcher = std::sync::Mutex::new(ProgressBatcher::new(Duration::from_millis(50)));
        let result = match std::panic::catch_unwind(AssertUnwindSafe(|| {
            ImageProcessor::batch_process(&files, &profile, &source_dir, &stop_flag, |event| {
                if let Ok(mut b) = batcher.lock() {
                    if let Some(batch) = b.record(event) {
                        let _ = app.emit("progress_update", &batch);
                    }
                }
            })
        })) {
            Ok(r) => {
                if let Ok(mut b) = batcher.lock() {
                    if let Some(batch) = b.flush() {
                        let _ = app.emit("progress_update", &batch);
                    }
                }
                r
            }
            Err(_) => {
                log::error!("Batch processing panicked");
                BatchResult {
                    total_files: files.len() as u32,
                    success: 0,
                    failed: files.len() as u32,
                    total_saved_bytes: 0,
                    failed_files: vec!["处理线程崩溃（panic），批处理已中止".to_string()],
                }
            }
        };
        let _ = app.emit("processing_complete", &result);
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_processing(state: tauri::State<'_, AppState>) -> Result<(), String> {
    if !state.is_processing.load(Ordering::SeqCst) {
        return Err("No processing in progress".to_string());
    }
    state.stop_flag.store(true, Ordering::SeqCst);
    Ok(())
}
