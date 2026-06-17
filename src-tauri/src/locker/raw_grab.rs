use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use parking_lot::RwLock;

use super::combo::{ComboResult, ComboTracker};
use super::device_manager::enumerate_keyboard_devices_with_config;
use super::engine::EventCallback;
use super::filter;
use crate::state::EngineState;

const RI_KEY_BREAK: u16 = 0x0001;
const RI_KEY_E0: u16 = 0x0002;

const VK_SHIFT: u16 = 0x10;
const VK_CONTROL: u16 = 0x11;
const VK_MENU: u16 = 0x12;
const VK_LSHIFT: u16 = 0xA0;
const VK_RSHIFT: u16 = 0xA1;
const VK_LCONTROL: u16 = 0xA2;
const VK_RCONTROL: u16 = 0xA3;
const VK_LMENU: u16 = 0xA4;
const VK_RMENU: u16 = 0xA5;
struct GrabCtx {
    state: Arc<RwLock<EngineState>>,
    cb: Option<EventCallback>,
    block_map: HashMap<isize, bool>,
    mods: HashSet<u32>,
    unlock_tracker: ComboTracker,
    lock_tracker: ComboTracker,
}

impl GrabCtx {
    fn handle_key(&mut self, vk: u16, scan: u16, flags: u16, hdev: isize) {
        let is_down = flags & RI_KEY_BREAK == 0;
        let specific_vk = normalize_vk(vk, scan, flags);

        if is_down {
            self.mods.insert(specific_vk as u32);
        } else {
            self.mods.remove(&(specific_vk as u32));
        }

        if !is_down {
            self.unlock_tracker.feed_key_release(specific_vk as u32);
            self.lock_tracker.feed_key_release(specific_vk as u32);
            self.do_inject(vk, scan, flags);
            return;
        }

        let config = { self.state.read().config.clone() };

        let mut combo_matched = false;
        let combo_result = self.unlock_tracker.feed_key_press(specific_vk as u32);
        let is_unlock_shortcut = config.unlock_combo.contains(&(specific_vk as u32));
        let is_lock_shortcut = config.lock_combo.contains(&(specific_vk as u32));

        if combo_result == ComboResult::Matched {
            self.unlock_tracker.reset();
            self.lock_tracker.reset();
            let was_locked = self.state.read().locked;
            if was_locked {
                self.state.write().locked = false;
                self.emit_lock_changed(false);
            }
            combo_matched = true;
        }

        if !combo_matched {
            let lr = self.lock_tracker.feed_key_press(specific_vk as u32);
            if lr == ComboResult::Matched {
                self.unlock_tracker.reset();
                self.lock_tracker.reset();
                let was_locked = self.state.read().locked;
                if !was_locked {
                    self.state.write().locked = true;
                    self.emit_lock_changed(true);
                }
                combo_matched = true;
            }
        }

        if combo_matched {
            return;
        }

        let is_shortcut = is_unlock_shortcut || is_lock_shortcut;
        let locked = self.state.read().locked;

        if locked {
            let should_block = self.block_map.get(&hdev).copied().unwrap_or(false);

            if is_shortcut {
                if !should_block {
                    self.do_inject(vk, scan, flags);
                    self.state.write().total_allowed += 1;
                } else {
                    self.state.write().total_blocked += 1;
                }
                return;
            }

            if should_block {
                self.state.write().total_blocked += 1;
                return;
            }

            let allow = filter::evaluate(&config, self.state.read().active_app.as_deref(), vk as u32, &self.mods);
            if allow {
                self.state.write().total_allowed += 1;
                self.do_inject(vk, scan, flags);
            } else {
                self.state.write().total_blocked += 1;
            }
            return;
        }

        self.state.write().total_allowed += 1;
        self.do_inject(vk, scan, flags);
    }

    fn do_inject(&self, vk: u16, _scan: u16, flags: u16) {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
            SendInput,
        };

        let is_up = flags & RI_KEY_BREAK != 0;
        let is_e0 = flags & RI_KEY_E0 != 0;

        let mut dw_flags = 0u32;
        if is_up {
            dw_flags |= KEYEVENTF_KEYUP;
        }
        if is_e0 {
            dw_flags |= KEYEVENTF_EXTENDEDKEY;
        }

        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: dw_flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        unsafe {
            SendInput(1, &input, size_of::<INPUT>() as i32);
        }
    }

    fn emit_lock_changed(&self, locked: bool) {
        if let Some(ref cb) = self.cb {
            cb("lock-state-changed", serde_json::json!({ "locked": locked }));
        }
    }
}

fn normalize_vk(vk: u16, scan: u16, flags: u16) -> u16 {
    let e0 = flags & RI_KEY_E0 != 0;
    match vk {
        VK_SHIFT if scan == 0x2A => VK_LSHIFT,
        VK_SHIFT if scan == 0x36 => VK_RSHIFT,
        VK_CONTROL if scan == 0x1D && !e0 => VK_LCONTROL,
        VK_CONTROL if scan == 0x1D && e0 => VK_RCONTROL,
        VK_MENU if scan == 0x38 && !e0 => VK_LMENU,
        VK_MENU if scan == 0x38 && e0 => VK_RMENU,
        _ => vk,
    }
}

fn read_raw_event(
    handle: windows_sys::Win32::UI::Input::HRAWINPUT,
) -> Option<(u16, u16, u16, isize)> {
    unsafe {
        let mut size: u32 = 0;
        let header_size = size_of::<windows_sys::Win32::UI::Input::RAWINPUTHEADER>() as u32;
        let rc = windows_sys::Win32::UI::Input::GetRawInputData(
            handle,
            windows_sys::Win32::UI::Input::RID_INPUT,
            null_mut(),
            &mut size,
            header_size,
        );
        if rc == u32::MAX || size == 0 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        let rc = windows_sys::Win32::UI::Input::GetRawInputData(
            handle,
            windows_sys::Win32::UI::Input::RID_INPUT,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
            &mut size,
            header_size,
        );
        if rc == u32::MAX {
            return None;
        }

        let raw = &*(buf.as_ptr() as *const windows_sys::Win32::UI::Input::RAWINPUT);
        if raw.header.dwType != windows_sys::Win32::UI::Input::RIM_TYPEKEYBOARD {
            return None;
        }

        let kb = raw.data.keyboard;
        Some((kb.VKey, kb.MakeCode, kb.Flags, raw.header.hDevice as isize))
    }
}

static GRAB_CTX: OnceLock<Mutex<Option<GrabCtx>>> = OnceLock::new();

fn grab_ctx() -> &'static Mutex<Option<GrabCtx>> {
    GRAB_CTX.get_or_init(|| Mutex::new(None))
}

unsafe extern "system" fn grab_wndproc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    w_param: windows_sys::Win32::Foundation::WPARAM,
    l_param: windows_sys::Win32::Foundation::LPARAM,
) -> windows_sys::Win32::Foundation::LRESULT {
    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_INPUT {
        if let Some((vk, scan, flags, hdev)) =
            read_raw_event(l_param as windows_sys::Win32::UI::Input::HRAWINPUT)
        {
            if let Ok(mut guard) = grab_ctx().lock() {
                if let Some(ref mut ctx) = *guard {
                    ctx.handle_key(vk, scan, flags, hdev);
                }
            }
        }
        return 0;
    }
    windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, w_param, l_param)
}

pub fn run_grab_loop(
    state: Arc<RwLock<EngineState>>,
    running: Arc<AtomicBool>,
    event_cb: Option<EventCallback>,
    grab_thread_id: Arc<parking_lot::Mutex<Option<u32>>>,
) {
    unsafe {
        use windows_sys::Win32::Foundation::GetLastError;
        use windows_sys::Win32::System::Threading::{GetCurrentThreadId, Sleep};
        use windows_sys::Win32::UI::Input::{RegisterRawInputDevices, RIDEV_NOLEGACY, RAWINPUTDEVICE};
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DispatchMessageW, PeekMessageW, RegisterClassW, HWND_MESSAGE,
            PM_REMOVE, WM_NULL, WNDCLASSW,
        };

        let class_name: Vec<u16> = "KLP_RawGrab\0".encode_utf16().collect();
        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(grab_wndproc),
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
                log::error!("Failed to register grab window class: {}", err);
                state.write().grab_active = false;
                return;
            }
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0, 0, 0, 0, 0,
            HWND_MESSAGE,
            null_mut(),
            null_mut(),
            null_mut(),
        );

        if hwnd.is_null() {
            log::error!("Failed to create grab message window");
            state.write().grab_active = false;
            return;
        }

        let rid = RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x06,
            dwFlags: RIDEV_NOLEGACY,
            hwndTarget: hwnd,
        };

        if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            log::error!("Failed to register raw input for grab (RIDEV_NOLEGACY)");
            state.write().grab_active = false;
            return;
        }

        let config = { state.read().config.clone() };
        let devices = enumerate_keyboard_devices_with_config(&config.keyboard_devices);
        let mut block_map = HashMap::new();
        for d in &devices {
            block_map.insert(d.handle, d.is_target && d.enabled);
        }

        let unlock_tracker = ComboTracker::new(config.unlock_combo.clone());
        let lock_tracker = ComboTracker::new(config.lock_combo.clone());

        {
            let mut guard = grab_ctx().lock().unwrap();
            *guard = Some(GrabCtx {
                state: state.clone(),
                cb: event_cb,
                block_map,
                mods: HashSet::new(),
                unlock_tracker,
                lock_tracker,
            });
        }

        grab_thread_id.lock().clone_from(&Some(GetCurrentThreadId()));
        state.write().grab_active = true;
        log::info!("Raw input grab loop started (RIDEV_NOLEGACY)");

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

        {
            let mut guard = grab_ctx().lock().unwrap();
            *guard = None;
        }

        log::info!("Raw input grab loop stopped");
    }
}
