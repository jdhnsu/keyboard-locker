#![cfg(target_os = "windows")]

//! Integration tests for Windows Raw Input keyboard API.

mod common;

use std::collections::HashMap;
use std::env;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use common::{
    create_message_window, enumerate_keyboard_devices, read_raw_input, KeyboardDevice,
    RawKeyboardEvent,
};
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Console::{
    SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT, CTRL_LOGOFF_EVENT,
    CTRL_SHUTDOWN_EVENT,
};
use windows_sys::Win32::System::Threading::{GetCurrentThreadId, Sleep};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::Input::HRAWINPUT;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, DispatchMessageW, PeekMessageW, PostThreadMessageW, PM_REMOVE, WM_INPUT,
    WM_NULL,
};

const RI_KEY_BREAK: u16 = 0x0001;
const RI_KEY_E0: u16 = 0x0002;

static DEVICE_MAP: LazyLock<std::sync::Mutex<HashMap<isize, (usize, KeyboardDevice)>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static STOP_LISTENER: AtomicBool = AtomicBool::new(false);
static LISTENER_THREAD_ID: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT
        | CTRL_SHUTDOWN_EVENT => {
            STOP_LISTENER.store(true, Ordering::SeqCst);
            let thread_id = LISTENER_THREAD_ID.load(Ordering::SeqCst);
            if thread_id != 0 {
                let _ = PostThreadMessageW(thread_id, WM_NULL, 0, 0);
            }
            1
        }
        _ => 0,
    }
}

unsafe extern "system" fn listener_window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            if let Some(event) = read_raw_input(l_param as HRAWINPUT) {
                print_raw_keyboard_event(&event);
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

fn listener_duration() -> Option<Duration> {
    env::var("KEYBOARD_LISTENER_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
}

fn register_devices(devices: Vec<KeyboardDevice>) {
    let mut map = DEVICE_MAP.lock().expect("device map lock poisoned");
    map.clear();

    for (index, device) in devices.into_iter().enumerate() {
        map.insert(device.handle, (index, device));
    }
}

fn format_device(index: usize, device: &KeyboardDevice) -> String {
    format!(
        "#{} {} (VID=0x{:04X} PID=0x{:04X} handle=0x{:X})",
        index + 1,
        device.name,
        device.vendor_id,
        device.product_id,
        device.handle
    )
}

fn event_state(flags: u16) -> &'static str {
    if flags & RI_KEY_BREAK != 0 {
        "UP"
    } else {
        "DOWN"
    }
}

fn print_raw_keyboard_event(event: &RawKeyboardEvent) {
    let device_text = DEVICE_MAP
        .lock()
        .expect("device map lock poisoned")
        .get(&event.device_handle)
        .map(|(index, device)| format_device(*index, device))
        .unwrap_or_else(|| format!("unknown (handle=0x{:X})", event.device_handle));

    println!(
        "[KEY] state={} key={} vk=0x{:04X} scan=0x{:04X} flags=0x{:04X} device={}",
        event_state(event.flags),
        key_name(event.vk_code, event.make_code, event.flags),
        event.vk_code,
        event.make_code,
        event.flags,
        device_text,
    );
}

fn key_name(vk_code: u16, make_code: u16, flags: u16) -> String {
    let vk = if vk_code != 0 {
        vk_code
    } else {
        unsafe { MapVirtualKeyW(make_code as u32, MAPVK_VSC_TO_VK_EX) as u16 }
    };
    let extended = flags & RI_KEY_E0 != 0;

    if let Some(name) = special_key_name(vk, make_code, extended) {
        return name;
    }

    match vk {
        0x30..=0x39 => ((vk as u8 - b'0') as char).to_string(),
        0x41..=0x5A => ((vk as u8 - b'A' + b'a') as char).to_string(),
        _ => {
            let ch = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_CHAR) & 0xffff };
            char::from_u32(ch as u32)
                .filter(|value| !value.is_control())
                .map(|value| value.to_string())
                .unwrap_or_else(|| format!("VK_{:04X}", vk))
        }
    }
}

fn special_key_name(vk: u16, make_code: u16, extended: bool) -> Option<String> {
    let name: String = match vk {
        VK_BACK => "Backspace".into(),
        VK_TAB => "Tab".into(),
        VK_CLEAR => "Clear".into(),
        VK_RETURN => if extended && make_code == 0x1c {
            "NumpadEnter"
        } else {
            "Enter"
        }
        .into(),
        VK_SHIFT if make_code == 0x2a => "LeftShift".into(),
        VK_SHIFT if make_code == 0x36 => "RightShift".into(),
        VK_CONTROL if extended => "RightCtrl".into(),
        VK_CONTROL => "LeftCtrl".into(),
        VK_MENU if extended => "RightAlt".into(),
        VK_MENU => "LeftAlt".into(),
        VK_PAUSE => "Pause".into(),
        VK_CAPITAL => "CapsLock".into(),
        VK_ESCAPE => "Esc".into(),
        VK_SPACE => "Space".into(),
        VK_PRIOR => "PageUp".into(),
        VK_NEXT => "PageDown".into(),
        VK_END => "End".into(),
        VK_HOME => "Home".into(),
        VK_LEFT => "Left".into(),
        VK_UP => "Up".into(),
        VK_RIGHT => "Right".into(),
        VK_DOWN => "Down".into(),
        VK_SELECT => "Select".into(),
        VK_PRINT => "Print".into(),
        VK_EXECUTE => "Execute".into(),
        VK_SNAPSHOT => "PrintScreen".into(),
        VK_INSERT => "Insert".into(),
        VK_DELETE => "Delete".into(),
        VK_HELP => "Help".into(),
        VK_LWIN => "LeftWin".into(),
        VK_RWIN => "RightWin".into(),
        VK_APPS => "Menu".into(),
        VK_SLEEP => "Sleep".into(),
        VK_NUMPAD0 => "Numpad0".into(),
        VK_NUMPAD1 => "Numpad1".into(),
        VK_NUMPAD2 => "Numpad2".into(),
        VK_NUMPAD3 => "Numpad3".into(),
        VK_NUMPAD4 => "Numpad4".into(),
        VK_NUMPAD5 => "Numpad5".into(),
        VK_NUMPAD6 => "Numpad6".into(),
        VK_NUMPAD7 => "Numpad7".into(),
        VK_NUMPAD8 => "Numpad8".into(),
        VK_NUMPAD9 => "Numpad9".into(),
        VK_MULTIPLY => "NumpadMultiply".into(),
        VK_ADD => "NumpadAdd".into(),
        VK_SEPARATOR => "Separator".into(),
        VK_SUBTRACT => "NumpadSubtract".into(),
        VK_DECIMAL => "NumpadDecimal".into(),
        VK_DIVIDE => "NumpadDivide".into(),
        VK_F1 => "F1".into(),
        VK_F2 => "F2".into(),
        VK_F3 => "F3".into(),
        VK_F4 => "F4".into(),
        VK_F5 => "F5".into(),
        VK_F6 => "F6".into(),
        VK_F7 => "F7".into(),
        VK_F8 => "F8".into(),
        VK_F9 => "F9".into(),
        VK_F10 => "F10".into(),
        VK_F11 => "F11".into(),
        VK_F12 => "F12".into(),
        VK_F13 => "F13".into(),
        VK_F14 => "F14".into(),
        VK_F15 => "F15".into(),
        VK_F16 => "F16".into(),
        VK_F17 => "F17".into(),
        VK_F18 => "F18".into(),
        VK_F19 => "F19".into(),
        VK_F20 => "F20".into(),
        VK_F21 => "F21".into(),
        VK_F22 => "F22".into(),
        VK_F23 => "F23".into(),
        VK_F24 => "F24".into(),
        VK_NUMLOCK => "NumLock".into(),
        VK_SCROLL => "ScrollLock".into(),
        VK_LSHIFT => "LeftShift".into(),
        VK_RSHIFT => "RightShift".into(),
        VK_LCONTROL => "LeftCtrl".into(),
        VK_RCONTROL => "RightCtrl".into(),
        VK_LMENU => "LeftAlt".into(),
        VK_RMENU => "RightAlt".into(),
        VK_BROWSER_BACK => "BrowserBack".into(),
        VK_BROWSER_FORWARD => "BrowserForward".into(),
        VK_BROWSER_REFRESH => "BrowserRefresh".into(),
        VK_BROWSER_STOP => "BrowserStop".into(),
        VK_BROWSER_SEARCH => "BrowserSearch".into(),
        VK_BROWSER_FAVORITES => "BrowserFavorites".into(),
        VK_BROWSER_HOME => "BrowserHome".into(),
        VK_VOLUME_MUTE => "VolumeMute".into(),
        VK_VOLUME_DOWN => "VolumeDown".into(),
        VK_VOLUME_UP => "VolumeUp".into(),
        VK_MEDIA_NEXT_TRACK => "MediaNextTrack".into(),
        VK_MEDIA_PREV_TRACK => "MediaPrevTrack".into(),
        VK_MEDIA_STOP => "MediaStop".into(),
        VK_MEDIA_PLAY_PAUSE => "MediaPlayPause".into(),
        VK_OEM_PLUS => "+".into(),
        VK_OEM_COMMA => ",".into(),
        VK_OEM_MINUS => "-".into(),
        VK_OEM_PERIOD => ".".into(),
        VK_OEM_2 => "/".into(),
        VK_OEM_3 => "`".into(),
        VK_OEM_4 => "[".into(),
        VK_OEM_5 => "\\".into(),
        VK_OEM_6 => "]".into(),
        VK_OEM_7 => "'".into(),
        VK_OEM_8 => "OEM8".into(),
        VK_OEM_1 => ";".into(),
        VK_OEM_102 => "OEM102".into(),
        VK_PROCESSKEY => "ProcessKey".into(),
        VK_PACKET => "Packet".into(),
        VK_ATTN => "Attn".into(),
        VK_CRSEL => "CrSel".into(),
        VK_EXSEL => "ExSel".into(),
        VK_EREOF => "EraseEOF".into(),
        VK_PLAY => "Play".into(),
        VK_ZOOM => "Zoom".into(),
        VK_NONCONVERT => "NonConvert".into(),
        VK_ACCEPT => "Accept".into(),
        VK_MODECHANGE => "ModeChange".into(),
        VK_PA1 => "PA1".into(),
        _ if vk >= VK_A && vk <= VK_Z => ((vk as u8 - b'A' + b'a') as char).to_string(),
        _ if vk >= VK_0 && vk <= VK_9 => ((vk as u8 - b'0') as char).to_string(),
        _ => return None,
    };

    Some(name)
}

fn run_message_loop(started_at: Instant) {
    unsafe {
        while !STOP_LISTENER.load(Ordering::SeqCst) {
            if let Some(duration) = listener_duration() {
                if started_at.elapsed() >= duration {
                    break;
                }
            }

            let mut msg = zeroed();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_NULL {
                    continue;
                }
                DispatchMessageW(&msg);
            }

            Sleep(10);
        }
    }
}

#[test]
fn test_enumerate_keyboard_devices() {
    println!("=== Test: Enumerate Keyboard Devices ===");
    println!("Thread id: {}", unsafe { GetCurrentThreadId() });

    let devices = enumerate_keyboard_devices();

    if devices.is_empty() {
        println!("No keyboard devices found via Raw Input.");
    } else {
        println!("Found {} keyboard device(s):", devices.len());
        for (index, device) in devices.iter().enumerate() {
            println!(
                "  [{}] handle=0x{:X} vid=0x{:04X} pid=0x{:04X} name={}",
                index + 1,
                device.handle,
                device.vendor_id,
                device.product_id,
                device.name
            );
        }
    }

    assert!(
        !devices.is_empty(),
        "Should find at least one keyboard device"
    );
}

#[test]
fn test_create_message_window() {
    println!("=== Test: Create Message Window ===");

    let hwnd = create_message_window(listener_window_proc);

    assert!(!hwnd.is_null(), "Failed to create message-only window");
    println!(
        "Message window created successfully with handle: 0x{:X}",
        hwnd as isize
    );
}

#[test]
fn test_register_raw_input() {
    println!("=== Test: Register Raw Input ===");

    let hwnd = create_message_window(listener_window_proc);
    assert!(!hwnd.is_null(), "Failed to create message window");

    let result = common::register_raw_input(hwnd);
    assert!(result, "Failed to register raw input device");

    println!("Raw input registered successfully");
    println!("  - Usage Page: 0x01 (Generic Desktop)");
    println!("  - Usage: 0x06 (Keyboard)");
    println!("  - Flags: RIDEV_EXINPUTSINK (receive input in background)");
}

#[test]
fn test_full_raw_input_setup() {
    println!("=== Test: Full Raw Input Setup ===");
    println!("This test verifies the complete setup flow:");
    println!("  1. Enumerate keyboard devices");
    println!("  2. Create message window");
    println!("  3. Register for raw input");

    let devices = enumerate_keyboard_devices();
    println!("\n[Step 1] Found {} keyboard device(s)", devices.len());
    assert!(!devices.is_empty(), "Should find at least one keyboard");

    let hwnd = create_message_window(listener_window_proc);
    println!("[Step 2] Message window created: 0x{:X}", hwnd as isize);
    assert!(!hwnd.is_null(), "Message window creation failed");

    let registered = common::register_raw_input(hwnd);
    println!(
        "[Step 3] Raw input registration: {}",
        if registered { "SUCCESS" } else { "FAILED" }
    );
    assert!(registered, "Raw input registration failed");

    println!("\nAll setup steps completed successfully!");
}

#[test]
#[ignore = "manual continuous listener; run with --ignored"]
fn test_continuous_keyboard_listener() {
    println!("=== Continuous Raw Input Keyboard Listener ===");
    println!("Thread id: {}", unsafe { GetCurrentThreadId() });
    println!("Press Ctrl+C to stop, or set KEYBOARD_LISTENER_SECONDS=<seconds> for auto-stop.");

    let devices = enumerate_keyboard_devices();
    assert!(
        !devices.is_empty(),
        "Should find at least one keyboard device"
    );

    register_devices(devices.clone());
    for (index, device) in devices.iter().enumerate() {
        println!(
            "Keyboard #{}: {} (VID=0x{:04X} PID=0x{:04X} handle=0x{:X})",
            index + 1,
            device.name,
            device.vendor_id,
            device.product_id,
            device.handle
        );
    }

    let hwnd = create_message_window(listener_window_proc);
    assert!(!hwnd.is_null(), "Failed to create message window");

    assert!(
        common::register_raw_input(hwnd),
        "Failed to register raw input device"
    );

    unsafe {
        LISTENER_THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);
        assert_ne!(
            SetConsoleCtrlHandler(Some(console_ctrl_handler), 1),
            0,
            "Failed to install console control handler"
        );
    }

    let started_at = Instant::now();
    println!("Listening for key presses. Output format: [KEY] state/key/vk/scan/device");

    run_message_loop(started_at);

    unsafe {
        let _ = SetConsoleCtrlHandler(None, 0);
    }

    println!("Keyboard listener stopped.");
}
