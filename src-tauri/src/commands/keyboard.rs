use tauri::State;

use crate::locker::rules::KeyRule;
use crate::commands::lifecycle::AppEngine;
use crate::commands::config::AppConfigStore;

#[tauri::command]
pub fn get_key_state(engine: State<'_, AppEngine>) -> Result<Vec<KeyRule>, String> {
    let state = engine.0.state.read();
    Ok(state.config.rules.clone())
}

#[tauri::command]
pub fn set_key_allowed(
    engine: State<'_, AppEngine>,
    store: State<'_, AppConfigStore>,
    key: u32,
    allowed: bool,
) -> Result<(), String> {
    let mut state = engine.0.state.write();
    if let Some(rule) = state.config.rules.iter_mut().find(|r| r.key == key) {
        rule.allowed = allowed;
    }
    let config = state.config.clone();
    drop(state);
    store.0.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reset_keys(engine: State<'_, AppEngine>) -> Result<(), String> {
    let mut state = engine.0.state.write();
    state.total_blocked = 0;
    state.total_allowed = 0;
    Ok(())
}