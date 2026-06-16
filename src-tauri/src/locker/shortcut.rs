use std::collections::HashMap;

use super::rules::Config;

pub fn combo_to_shortcut_str(combo: &[u32]) -> String {
    let vk_map = build_vk_to_shortcut_map();
    let mut parts: Vec<String> = Vec::new();
    let mut main_key: Option<String> = None;

    for &vk in combo {
        if let Some(label) = vk_map.get(&vk) {
            match vk {
                0xA0 | 0xA1 => parts.push("Shift".to_string()),
                0xA2 | 0xA3 => parts.push("Ctrl".to_string()),
                0xA4 | 0xA5 => parts.push("Alt".to_string()),
                0x5B | 0x5C => parts.push("Super".to_string()),
                _ => main_key = Some(label.clone()),
            }
        } else {
            main_key = Some(format!("0x{:02X}", vk));
        }
    }

    if let Some(key) = main_key {
        parts.push(key);
    }

    parts.join("+")
}

fn build_vk_to_shortcut_map() -> HashMap<u32, String> {
    let mut m = HashMap::new();

    m.insert(0x08, "Backspace".to_string());
    m.insert(0x09, "Tab".to_string());
    m.insert(0x0D, "Enter".to_string());
    m.insert(0x13, "Pause".to_string());
    m.insert(0x14, "CapsLock".to_string());
    m.insert(0x1B, "Escape".to_string());
    m.insert(0x20, "Space".to_string());
    m.insert(0x21, "PageUp".to_string());
    m.insert(0x22, "PageDown".to_string());
    m.insert(0x23, "End".to_string());
    m.insert(0x24, "Home".to_string());
    m.insert(0x25, "ArrowLeft".to_string());
    m.insert(0x26, "ArrowUp".to_string());
    m.insert(0x27, "ArrowRight".to_string());
    m.insert(0x28, "ArrowDown".to_string());
    m.insert(0x2D, "Insert".to_string());
    m.insert(0x2E, "Delete".to_string());

    for i in 0..=9 {
        m.insert(0x30 + i, char::from_digit(i, 10).unwrap().to_string());
    }
    for c in 'A'..='Z' {
        m.insert(c as u32, format!("Key{}", c));
    }

    m.insert(0x5B, "Super".to_string());
    m.insert(0x5C, "Super".to_string());

    for i in 0..=9 {
        m.insert(0x60 + i, format!("Num{}", i));
    }
    m.insert(0x6A, "NumMultiply".to_string());
    m.insert(0x6B, "NumAdd".to_string());
    m.insert(0x6D, "NumSubtract".to_string());
    m.insert(0x6E, "NumDecimal".to_string());
    m.insert(0x6F, "NumDivide".to_string());

    for i in 1..=12 {
        m.insert(0x70 + i as u32 - 1, format!("F{}", i));
    }

    m.insert(0x90, "NumLock".to_string());
    m.insert(0x91, "ScrollLock".to_string());

    m.insert(0xA0, "Shift".to_string());
    m.insert(0xA1, "Shift".to_string());
    m.insert(0xA2, "Ctrl".to_string());
    m.insert(0xA3, "Ctrl".to_string());
    m.insert(0xA4, "Alt".to_string());
    m.insert(0xA5, "Alt".to_string());

    m.insert(0xBA, ";".to_string());
    m.insert(0xBB, "=".to_string());
    m.insert(0xBC, ",".to_string());
    m.insert(0xBD, "-".to_string());
    m.insert(0xBE, ".".to_string());
    m.insert(0xBF, "/".to_string());
    m.insert(0xC0, "`".to_string());
    m.insert(0xDB, "[".to_string());
    m.insert(0xDC, "\\".to_string());
    m.insert(0xDD, "]".to_string());
    m.insert(0xDE, "'".to_string());

    m
}

pub fn register_global_shortcuts(
    app: &tauri::AppHandle,
    config: &Config,
    engine: &crate::locker::engine::Engine,
) {
    use tauri::Emitter;
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

    let global_shortcut = app.global_shortcut();

    let _ = global_shortcut.unregister_all();

    crate::locker::engine::update_hook_shortcut_keys(&config.unlock_combo, &config.lock_combo);

    let unlock_str = combo_to_shortcut_str(&config.unlock_combo);
    let lock_str = combo_to_shortcut_str(&config.lock_combo);

    if let Ok(shortcut) = unlock_str.parse::<Shortcut>() {
        let engine_unlock = engine.state.clone();
        let app_unlock = app.clone();
        let _ = global_shortcut.on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let mut s = engine_unlock.write();
                if s.locked {
                    s.locked = false;
                    s.combo_tracker.reset();
                    s.lock_combo_tracker.reset();
                    drop(s);
                    let _ = app_unlock.emit("lock-state-changed", serde_json::json!({"locked": false}));
                }
            }
        });
        log::info!("Registered unlock shortcut: {}", unlock_str);
    } else {
        log::warn!("Failed to parse unlock shortcut: {}", unlock_str);
    }

    if let Ok(shortcut) = lock_str.parse::<Shortcut>() {
        let engine_lock = engine.state.clone();
        let app_lock = app.clone();
        let _ = global_shortcut.on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let mut s = engine_lock.write();
                if !s.locked {
                    s.locked = true;
                    s.combo_tracker.reset();
                    s.lock_combo_tracker.reset();
                    drop(s);
                    let _ = app_lock.emit("lock-state-changed", serde_json::json!({"locked": true}));
                }
            }
        });
        log::info!("Registered lock shortcut: {}", lock_str);
    } else {
        log::warn!("Failed to parse lock shortcut: {}", lock_str);
    }
}