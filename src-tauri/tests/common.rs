#![cfg(target_os = "windows")]

//! Common utilities and helpers for Windows keyboard tests.

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Input::{
    GetRawInputData, GetRawInputDeviceInfoW, GetRawInputDeviceList, RegisterRawInputDevices,
    HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTDEVICELIST, RAWINPUTHEADER, RIDEV_EXINPUTSINK,
    RIDI_DEVICENAME, RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, HWND_MESSAGE, WM_INPUT, WNDCLASSW,
};

/// Parse VID and PID from a device path string like "\\?\HID#VID_1234&PID_5678#..."
pub fn parse_vid_pid(path: &str) -> (u16, u16) {
    let vid = path
        .split("VID_")
        .nth(1)
        .and_then(|s| s.get(..4))
        .and_then(|s| u16::from_str_radix(s, 16).ok())
        .unwrap_or(0);
    let pid = path
        .split("PID_")
        .nth(1)
        .and_then(|s| s.get(..4))
        .and_then(|s| u16::from_str_radix(s, 16).ok())
        .unwrap_or(0);
    (vid, pid)
}

/// Represents a keyboard device discovered via Raw Input API
#[derive(Debug, Clone)]
pub struct KeyboardDevice {
    pub handle: isize,
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
}

/// Represents a raw keyboard input event
#[derive(Debug, Clone, Copy)]
pub struct RawKeyboardEvent {
    pub device_handle: isize,
    pub vk_code: u16,
    pub make_code: u16,
    pub flags: u16,
}

/// Window procedure for message-only window to receive WM_INPUT
pub unsafe extern "system" fn raw_input_window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            if let Some(event) = read_raw_input(l_param as HRAWINPUT) {
                println!(
                    "[INPUT] device=0x{:X} vk=0x{:02X} make_code=0x{:02X} flags=0x{:04X}",
                    event.device_handle, event.vk_code, event.make_code, event.flags
                );
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

/// Read raw keyboard input data from a HRAWINPUT handle
pub fn read_raw_input(handle: HRAWINPUT) -> Option<RawKeyboardEvent> {
    unsafe {
        let mut size: u32 = 0;
        let header_size = size_of::<RAWINPUTHEADER>() as u32;
        let rc = GetRawInputData(handle, RID_INPUT, null_mut(), &mut size, header_size);
        if rc == u32::MAX || size == 0 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        let rc = GetRawInputData(
            handle,
            RID_INPUT,
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
            header_size,
        );
        if rc == u32::MAX {
            return None;
        }

        let raw = &*(buf.as_ptr() as *const RAWINPUT);
        if raw.header.dwType != RIM_TYPEKEYBOARD {
            return None;
        }

        let kb = raw.data.keyboard;
        Some(RawKeyboardEvent {
            device_handle: raw.header.hDevice as isize,
            vk_code: kb.VKey,
            make_code: kb.MakeCode,
            flags: kb.Flags,
        })
    }
}

/// Enumerate all keyboard devices using Raw Input API
pub fn enumerate_keyboard_devices() -> Vec<KeyboardDevice> {
    unsafe {
        let mut count: u32 = 0;
        let list_result = GetRawInputDeviceList(
            null_mut(),
            &mut count,
            size_of::<RAWINPUTDEVICELIST>() as u32,
        );
        if list_result == u32::MAX || count == 0 {
            eprintln!("failed to query raw input device count: {}", GetLastError());
            return Vec::new();
        }

        let mut devices = vec![
            RAWINPUTDEVICELIST {
                hDevice: null_mut(),
                dwType: 0,
            };
            count as usize
        ];
        let actual = GetRawInputDeviceList(
            devices.as_mut_ptr(),
            &mut count,
            size_of::<RAWINPUTDEVICELIST>() as u32,
        );
        if actual == u32::MAX {
            eprintln!("failed to enumerate raw input devices: {}", GetLastError());
            return Vec::new();
        }

        let mut result = Vec::new();

        for device in devices.into_iter().take(count as usize) {
            if device.dwType != RIM_TYPEKEYBOARD {
                continue;
            }

            let mut name_len: u32 = 0;
            let name_query =
                GetRawInputDeviceInfoW(device.hDevice, RIDI_DEVICENAME, null_mut(), &mut name_len);
            if name_query == u32::MAX || name_len == 0 {
                continue;
            }

            let mut name_buf = vec![0u16; name_len as usize];
            let name_result = GetRawInputDeviceInfoW(
                device.hDevice,
                RIDI_DEVICENAME,
                name_buf.as_mut_ptr() as *mut c_void,
                &mut name_len,
            );
            if name_result == u32::MAX {
                continue;
            }

            let name = String::from_utf16_lossy(&name_buf[..(name_len as usize).saturating_sub(1)]);
            let (vendor_id, product_id) = parse_vid_pid(&name);

            result.push(KeyboardDevice {
                handle: device.hDevice as isize,
                name,
                vendor_id,
                product_id,
            });
        }

        result
    }
}

/// Create a message-only window for receiving WM_INPUT messages
pub fn create_message_window(
    window_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
) -> HWND {
    unsafe {
        let class_name: Vec<u16> = "TestKbdRawInput\0".encode_utf16().collect();
        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            hInstance: null_mut(),
            lpszClassName: class_name.as_ptr(),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: std::ptr::null(),
        };

        let atom = RegisterClassW(&wnd_class);
        if atom == 0 {
            let err = GetLastError();
            if err != 1410 {
                // ERROR_CLASS_ALREADY_EXISTS
                eprintln!("failed to register window class: {}", err);
                return null_mut();
            }
        }

        CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    }
}

/// Register for raw input from keyboard devices
pub fn register_raw_input(target_hwnd: HWND) -> bool {
    unsafe {
        let rid = RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x06,
            dwFlags: RIDEV_EXINPUTSINK,
            hwndTarget: target_hwnd,
        };

        RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) != 0
    }
}
