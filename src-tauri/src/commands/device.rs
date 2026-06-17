use parking_lot::Mutex;
use tauri::State;

use crate::commands::config::AppConfigStore;
use crate::commands::lifecycle::AppEngine;
use crate::locker::device_manager::{
    enumerate_keyboard_devices_with_config, DeviceIdentifier, KeyboardDeviceConfig,
    KeyboardDeviceInfo,
};

pub struct AppDeviceIdentifier(pub Mutex<DeviceIdentifier>);

#[tauri::command]
pub fn enumerate_keyboards(
    engine: State<'_, AppEngine>,
) -> Result<Vec<KeyboardDeviceInfo>, String> {
    let configs = {
        let state = engine.0.state.read();
        state.config.keyboard_devices.clone()
    };
    Ok(enumerate_keyboard_devices_with_config(&configs))
}

#[tauri::command]
pub fn update_keyboard_device(
    engine: State<'_, AppEngine>,
    store: State<'_, AppConfigStore>,
    device: KeyboardDeviceConfig,
) -> Result<(), String> {
    let mut state = engine.0.state.write();
    let devices = &mut state.config.keyboard_devices;
    if let Some(existing) = devices.iter_mut().find(|d| d.instance_id == device.instance_id) {
        existing.alias = device.alias;
        existing.enabled = device.enabled;
        existing.is_target = device.is_target;
    } else {
        devices.push(device);
    }
    let config = state.config.clone();
    drop(state);
    store.0.save(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn identify_keyboard_start(
    app: tauri::AppHandle,
    identifier: State<'_, AppDeviceIdentifier>,
) -> Result<(), String> {
    let mut guard = identifier.0.lock();
    if guard.is_running() {
        return Err("已在识别中".to_string());
    }
    guard.start(app);
    Ok(())
}

#[tauri::command]
pub fn identify_keyboard_stop(
    identifier: State<'_, AppDeviceIdentifier>,
) -> Result<(), String> {
    let mut guard = identifier.0.lock();
    guard.stop();
    Ok(())
}
