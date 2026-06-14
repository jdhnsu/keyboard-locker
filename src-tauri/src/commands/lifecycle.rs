use tauri::State;

use crate::locker::engine::EngineSnapshot;
use crate::platform::{self, PlatformExtras};

pub struct AppEngine(pub crate::locker::engine::Engine);

#[tauri::command]
pub fn lock(engine: State<'_, AppEngine>) -> Result<(), String> {
    engine.0.lock();
    Ok(())
}

#[tauri::command]
pub fn unlock(engine: State<'_, AppEngine>) -> Result<(), String> {
    engine.0.unlock();
    Ok(())
}

#[tauri::command]
pub fn toggle_lock(engine: State<'_, AppEngine>) -> Result<bool, String> {
    Ok(engine.0.toggle())
}

#[tauri::command]
pub fn get_status(engine: State<'_, AppEngine>) -> Result<EngineSnapshot, String> {
    Ok(engine.0.get_snapshot())
}

#[tauri::command]
pub fn check_permissions() -> String {
    let p = platform::create_platform();
    match p.check_permissions() {
        platform::PermissionStatus::Granted => "granted".to_string(),
        platform::PermissionStatus::Denied { reason, .. } => reason,
    }
}

#[tauri::command]
pub fn open_permission_settings() {
    let p = platform::create_platform();
    p.open_permission_settings();
}