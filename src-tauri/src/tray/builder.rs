use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

use crate::commands::lifecycle::AppEngine;

const TRAY_ICON_DEFAULT: tauri::image::Image<'_> = tauri::include_image!("icons/tray-icon.png");
const TRAY_ICON_LOCKED: tauri::image::Image<'_> = tauri::include_image!("icons/tray-icon-locked.png");

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
    }
    if let Some(engine) = app.try_state::<AppEngine>() {
        engine.0.set_lightweight_mode(false);
    }
}

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_item = MenuItemBuilder::with_id("toggle", "锁定键盘").build(app)?;
    let show_item = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出应用").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&toggle_item)
        .item(&show_item)
        .item(&sep)
        .item(&quit_item)
        .build()?;

    let _tray = TrayIconBuilder::with_id("main")
        .tooltip("Keyboard Locker - 后台运行中")
        .icon(TRAY_ICON_DEFAULT.clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => {
                if let Some(engine) = app.try_state::<AppEngine>() {
                    let locked = engine.0.toggle();
                    if let Some(tray) = app.tray_by_id("main") {
                        let icon = if locked {
                            TRAY_ICON_LOCKED.clone()
                        } else {
                            TRAY_ICON_DEFAULT.clone()
                        };
                        let _ = tray.set_icon(Some(icon));
                        let tooltip = if locked {
                            "Keyboard Locker - 键盘已锁定"
                        } else {
                            "Keyboard Locker - 后台运行中"
                        };
                        let _ = tray.set_tooltip(Some(tooltip));
                    }
                    let msg = if locked { "键盘已锁定" } else { "键盘已解锁" };
                    let _ = app.emit("tray-notification", serde_json::json!({"message": msg}));
                }
            }
            "show" => {
                show_main_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    let app = tray.app_handle();
                    show_main_window(app);
                }
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    let app = tray.app_handle();
                    show_main_window(app);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}