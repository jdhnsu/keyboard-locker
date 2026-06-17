use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardDeviceConfig {
    pub instance_id: String,
    pub alias: String,
    pub enabled: bool,
    pub is_target: bool,
}

impl Default for KeyboardDeviceConfig {
    fn default() -> Self {
        KeyboardDeviceConfig {
            instance_id: String::new(),
            alias: String::new(),
            enabled: true,
            is_target: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyboardDeviceInfo {
    pub instance_id: String,
    pub alias: String,
    pub enabled: bool,
    pub is_target: bool,
    #[serde(skip)]
    pub handle: isize,
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
}

pub fn compute_instance_id(device_name: &str) -> String {
    device_name.trim_end_matches('\0').to_string()
}

pub fn merge_device_config(
    configs: &[KeyboardDeviceConfig],
    handle: isize,
    name: String,
    vendor_id: u16,
    product_id: u16,
) -> KeyboardDeviceInfo {
    let instance_id = compute_instance_id(&name);
    let base = configs
        .iter()
        .find(|c| c.instance_id == instance_id);

    match base {
        Some(c) => KeyboardDeviceInfo {
            instance_id,
            alias: c.alias.clone(),
            enabled: c.enabled,
            is_target: c.is_target,
            handle,
            name,
            vendor_id,
            product_id,
        },
        None => KeyboardDeviceInfo {
            instance_id,
            alias: String::new(),
            enabled: true,
            is_target: false,
            handle,
            name,
            vendor_id,
            product_id,
        },
    }
}

#[cfg(target_os = "windows")]
pub fn enumerate_keyboard_devices_with_config(
    configs: &[KeyboardDeviceConfig],
) -> Vec<KeyboardDeviceInfo> {
    internal_enumerate(configs)
}

#[cfg(not(target_os = "windows"))]
pub fn enumerate_keyboard_devices_with_config(
    _configs: &[KeyboardDeviceConfig],
) -> Vec<KeyboardDeviceInfo> {
    Vec::new()
}

#[cfg(target_os = "windows")]
fn internal_enumerate(configs: &[KeyboardDeviceConfig]) -> Vec<KeyboardDeviceInfo> {
    unsafe {
        use std::ffi::c_void;
        use std::mem::size_of;
        use std::ptr::null_mut;
        use windows_sys::Win32::UI::Input::{
            GetRawInputDeviceInfoW, GetRawInputDeviceList, RIDI_DEVICENAME, RIM_TYPEKEYBOARD,
            RAWINPUTDEVICELIST,
        };

        let mut count: u32 = 0;
        let list_result =
            GetRawInputDeviceList(null_mut(), &mut count, size_of::<RAWINPUTDEVICELIST>() as u32);
        if list_result == u32::MAX || count == 0 {
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

            let name =
                String::from_utf16_lossy(&name_buf[..(name_len as usize).saturating_sub(1)]);
            let (vendor_id, product_id) = parse_vid_pid(&name);

            result.push(merge_device_config(configs, device.hDevice as isize, name, vendor_id, product_id));
        }

        result
    }
}

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

// ---------------------------------------------------------------------------
// Tap-to-identify: Raw Input listener
// ---------------------------------------------------------------------------

struct IdentifierState {
    app_handle: tauri::AppHandle,
    device_map: HashMap<isize, String>,
}

static IDENTIFIER_STATE: OnceLock<Mutex<Option<IdentifierState>>> = OnceLock::new();

fn identifier_state() -> &'static Mutex<Option<IdentifierState>> {
    IDENTIFIER_STATE.get_or_init(|| Mutex::new(None))
}

pub struct DeviceIdentifier {
    running: Arc<AtomicBool>,
    thread_id: Arc<AtomicU32>,
    thread_join: Option<std::thread::JoinHandle<()>>,
}

impl DeviceIdentifier {
    pub fn new() -> Self {
        DeviceIdentifier {
            running: Arc::new(AtomicBool::new(false)),
            thread_id: Arc::new(AtomicU32::new(0)),
            thread_join: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn start(&mut self, app_handle: tauri::AppHandle) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let running = self.running.clone();
        let thread_id = self.thread_id.clone();

        let handle = std::thread::Builder::new()
            .name("keylock-identifier".into())
            .spawn(move || {
                run_identifier(app_handle, running, thread_id);
            })
            .expect("failed to spawn identifier thread");

        self.thread_join = Some(handle);
    }

    pub fn stop(&mut self) {
        if !self.running.swap(false, Ordering::SeqCst) {
            return;
        }

        let tid = self.thread_id.load(Ordering::SeqCst);
        if tid != 0 {
            #[cfg(target_os = "windows")]
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                    tid,
                    windows_sys::Win32::UI::WindowsAndMessaging::WM_NULL,
                    0,
                    0,
                );
            }
        }

        if let Some(handle) = self.thread_join.take() {
            let _ = handle.join();
        }

        if let Ok(mut guard) = identifier_state().lock() {
            *guard = None;
        }
    }
}

impl Drop for DeviceIdentifier {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn identifier_wndproc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    w_param: windows_sys::Win32::Foundation::WPARAM,
    l_param: windows_sys::Win32::Foundation::LPARAM,
) -> windows_sys::Win32::Foundation::LRESULT {
    use windows_sys::Win32::UI::Input::HRAWINPUT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{DefWindowProcW, WM_INPUT};

    if msg == WM_INPUT {
        let event = read_raw_identifier_input(l_param as HRAWINPUT);
        if let Some(ev) = event {
            if let Ok(guard) = identifier_state().lock() {
                if let Some(ref state) = *guard {
                    if let Some(instance_id) = state.device_map.get(&ev.device_handle) {
                        let _ = state.app_handle.emit(
                            "keyboard-tapped",
                            serde_json::json!({
                                "instance_id": instance_id,
                                "handle": ev.device_handle,
                            }),
                        );
                    }
                }
            }
        }
        return 0;
    }

    DefWindowProcW(hwnd, msg, w_param, l_param)
}

#[cfg(target_os = "windows")]
struct RawInputIdEvent {
    device_handle: isize,
}

#[cfg(target_os = "windows")]
fn read_raw_identifier_input(
    handle: windows_sys::Win32::UI::Input::HRAWINPUT,
) -> Option<RawInputIdEvent> {
    unsafe {
        use std::ffi::c_void;
        use std::mem::size_of;
        use std::ptr::null_mut;
        use windows_sys::Win32::UI::Input::{
            GetRawInputData, RAWINPUT, RAWINPUTHEADER, RID_INPUT, RIM_TYPEKEYBOARD,
        };

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

        Some(RawInputIdEvent {
            device_handle: raw.header.hDevice as isize,
        })
    }
}

#[cfg(target_os = "windows")]
fn run_identifier(
    app_handle: tauri::AppHandle,
    running: Arc<AtomicBool>,
    thread_id: Arc<AtomicU32>,
) {
    use std::mem::size_of;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::System::Threading::{GetCurrentThreadId, Sleep};
    use windows_sys::Win32::UI::Input::{RegisterRawInputDevices, RIDEV_EXINPUTSINK, RAWINPUTDEVICE};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, PeekMessageW, RegisterClassW, HWND_MESSAGE, PM_REMOVE,
        WM_NULL, WNDCLASSW,
    };

    unsafe {
        let class_name: Vec<u16> = "KLP_DevId\0".encode_utf16().collect();

        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(identifier_wndproc),
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
                log::error!("Failed to register identifier window class: {}", err);
                running.store(false, Ordering::SeqCst);
                return;
            }
        }

        let hwnd = CreateWindowExW(
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
        );

        if hwnd.is_null() {
            log::error!("Failed to create identifier message window");
            running.store(false, Ordering::SeqCst);
            return;
        }

        let rid = RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x06,
            dwFlags: RIDEV_EXINPUTSINK,
            hwndTarget: hwnd,
        };

        if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            log::error!("Failed to register raw input for identifier");
            running.store(false, Ordering::SeqCst);
            return;
        }

        let configs = Vec::new();
        let devices = internal_enumerate(&configs);
        let mut map = HashMap::new();
        for dev in &devices {
            map.insert(dev.handle, dev.instance_id.clone());
        }

        if let Ok(mut guard) = identifier_state().lock() {
            *guard = Some(IdentifierState {
                app_handle: app_handle.clone(),
                device_map: map,
            });
        }

        thread_id.store(GetCurrentThreadId(), Ordering::SeqCst);
        log::info!("Device identifier listener started");

        let mut msg = std::mem::zeroed();
        while running.load(Ordering::SeqCst) {
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_NULL {
                    continue;
                }
                DispatchMessageW(&msg);
            }
            Sleep(10);
        }

        log::info!("Device identifier listener stopped");
    }
}

#[cfg(not(target_os = "windows"))]
fn run_identifier(
    _app_handle: tauri::AppHandle,
    running: Arc<AtomicBool>,
    thread_id: Arc<AtomicU32>,
) {
    thread_id.store(0, Ordering::SeqCst);
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
