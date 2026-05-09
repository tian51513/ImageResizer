use crate::config::*;
use crate::processor::ImageProcessor;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<FileMetadata>, String> {
    crate::scanner::scan_directory(&path)
}

#[tauri::command]
pub async fn get_profiles(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Profile>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.config_manager.load_profiles())
}

#[tauri::command]
pub async fn save_profile(
    state: tauri::State<'_, Mutex<AppState>>,
    profile: Profile,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let mut profiles = state.config_manager.load_profiles();
    if let Some(pos) = profiles.iter().position(|p| p.name == profile.name) {
        profiles[pos] = profile;
    } else {
        profiles.push(profile);
    }
    state.config_manager.save_profiles(&profiles)
}

#[tauri::command]
pub async fn delete_profile(
    state: tauri::State<'_, Mutex<AppState>>,
    name: String,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
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
    state: tauri::State<'_, Mutex<AppState>>,
    files: Vec<FileMetadata>,
    profile: Profile,
    source_dir: String,
) -> Result<(), String> {
    {
        let state = state.lock().map_err(|e| e.to_string())?;
        if state.is_processing.load(Ordering::Relaxed) {
            return Err("Processing is already in progress".to_string());
        }
        state.is_processing.store(true, Ordering::Relaxed);
        state.stop_flag.store(false, Ordering::Relaxed);
    }

    let stop_flag = {
        let state = state.lock().map_err(|e| e.to_string())?;
        Arc::clone(&state.stop_flag)
    };
    let is_processing = {
        let state = state.lock().map_err(|e| e.to_string())?;
        Arc::clone(&state.is_processing)
    };

    tauri::async_runtime::spawn_blocking(move || {
        let result = ImageProcessor::batch_process(
            &files,
            &profile,
            &source_dir,
            &stop_flag,
            |event| {
                let _ = app.emit("progress_update", &event);
            },
        );

        let _ = app.emit("processing_complete", &result);
        is_processing.store(false, Ordering::Relaxed);
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_processing(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    if !state.is_processing.load(Ordering::Relaxed) {
        return Err("No processing in progress".to_string());
    }
    state.stop_flag.store(true, Ordering::Relaxed);
    Ok(())
}
