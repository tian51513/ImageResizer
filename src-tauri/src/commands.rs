// Tauri commands will be implemented in Task 5

#[tauri::command]
pub fn scan_directory() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn get_profiles() -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
pub fn save_profile() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn delete_profile() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn start_processing() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn stop_processing() -> Result<(), String> {
    Ok(())
}
