mod commands;
mod config;
mod locker;
mod platform;
mod state;
mod tray;

use commands::config::AppConfigStore;
use commands::lifecycle::AppEngine;
use config::ConfigStore;
use locker::engine::{Engine, EventCallback};
use tauri::Manager;
use tray::create_tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = ConfigStore::new();
    let config = store.load().unwrap_or_default();
    let engine = Engine::new(config.clone());

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppEngine(engine))
        .manage(AppConfigStore(store))
        .setup(|app| {
            use tauri::Emitter;

            let handle = app.handle().clone();

            let cb: EventCallback = std::sync::Arc::new(move |event, payload| {
                let _ = handle.emit(event, payload);
            });

            if let Some(app_engine) = app.try_state::<AppEngine>() {
                app_engine.0.set_event_callback(cb);
                app_engine.0.start_grab();
                app_engine.0.start_foreground_tracker();

                let config = {
                    let s = app_engine.0.state.read();
                    s.config.clone()
                };
                locker::shortcut::register_global_shortcuts(app.handle(), &config, &app_engine.0);
            }

            create_tray(app.handle())?;

            let window = app.get_webview_window("main").expect("main window not found");
            let app_handle_close = app.handle().clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Some(w) = app_handle_close.get_webview_window("main") {
                        let _ = w.hide();
                    }
                    if let Some(engine) = app_handle_close.try_state::<AppEngine>() {
                        engine.0.set_lightweight_mode(true);
                    }
                    log::info!("Window hidden to tray (close prevented)");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::lifecycle::lock,
            commands::lifecycle::unlock,
            commands::lifecycle::toggle_lock,
            commands::lifecycle::restart_grab,
            commands::lifecycle::get_status,
            commands::lifecycle::check_permissions,
            commands::lifecycle::fix_permissions,
            commands::lifecycle::open_permission_settings,
            commands::config::get_config,
            commands::config::update_config,
            commands::config::add_rule,
            commands::config::remove_rule,
            commands::config::add_app_rule,
            commands::config::remove_app_rule,
            commands::config::reset_config,
            commands::keyboard::get_key_state,
            commands::keyboard::set_key_allowed,
            commands::keyboard::reset_keys,
            commands::keyboard::set_unlock_combo,
            commands::keyboard::set_lock_combo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}