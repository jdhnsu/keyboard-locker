use tauri::State;

use crate::commands::lifecycle::AppEngine;
use crate::config::ConfigStore;
use crate::locker::rules::{AppRule, Config, KeyRule};

pub struct AppConfigStore(pub ConfigStore);

#[tauri::command]
pub fn get_config(engine: State<'_, AppEngine>) -> Result<Config, String> {
    let state = engine.0.state.read();
    Ok(state.config.clone())
}

#[tauri::command]
pub fn update_config(
    engine: State<'_, AppEngine>,
    store: State<'_, AppConfigStore>,
    config: Config,
) -> Result<(), String> {
    engine.0.update_config(config.clone());
    store.0.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_rule(
    engine: State<'_, AppEngine>,
    store: State<'_, AppConfigStore>,
    key: u32,
    label: String,
    allowed: bool,
) -> Result<(), String> {
    let mut state = engine.0.state.write();
    state.config.rules.retain(|r| r.key != key);
    state.config.rules.push(KeyRule {
        key,
        label,
        allowed,
        modifiers: None,
    });
    let config = state.config.clone();
    drop(state);
    store.0.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_rule(
    engine: State<'_, AppEngine>,
    store: State<'_, AppConfigStore>,
    key: u32,
) -> Result<(), String> {
    let mut state = engine.0.state.write();
    state.config.rules.retain(|r| r.key != key);
    let config = state.config.clone();
    drop(state);
    store.0.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_app_rule(
    engine: State<'_, AppEngine>,
    store: State<'_, AppConfigStore>,
    app_rule: AppRule,
) -> Result<(), String> {
    let mut state = engine.0.state.write();
    state
        .config
        .app_rules
        .retain(|r| r.process_names != app_rule.process_names);
    state.config.app_rules.push(app_rule);
    let config = state.config.clone();
    drop(state);
    store.0.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_app_rule(
    engine: State<'_, AppEngine>,
    store: State<'_, AppConfigStore>,
    process_names: Vec<String>,
) -> Result<(), String> {
    let mut state = engine.0.state.write();
    state
        .config
        .app_rules
        .retain(|r| r.process_names != process_names);
    let config = state.config.clone();
    drop(state);
    store.0.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reset_config(
    engine: State<'_, AppEngine>,
    store: State<'_, AppConfigStore>,
) -> Result<Config, String> {
    let config = store.0.restore_defaults().map_err(|e| e.to_string())?;
    engine.0.update_config(config.clone());
    Ok(config)
}
