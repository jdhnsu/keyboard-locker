use std::collections::{HashMap, HashSet};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use parking_lot::RwLock;

use super::combo::{ComboResult, ComboTracker};
use super::device_manager::enumerate_keyboard_devices_with_config;
use super::engine::EventCallback;
use super::filter;
use crate::state::EngineState;

const VK_SHIFT: u16 = 0x10;
const VK_CONTROL: u16 = 0x11;
const VK_MENU: u16 = 0x12;
const VK_LSHIFT: u16 = 0xA0;
const VK_RSHIFT: u16 = 0xA1;
const VK_LCONTROL: u16 = 0xA2;
const VK_RCONTROL: u16 = 0xA3;
const VK_LMENU: u16 = 0xA4;
const VK_RMENU: u16 = 0xA5;
const VK_LWIN: u16 = 0x5B;
const VK_RWIN: u16 = 0x5C;

struct GrabCtx {
    state: Arc<RwLock<EngineState>>,
    cb: Option<EventCallback>,
    block_map: HashMap<String, bool>,
    mods: HashSet<u32>,
    unlock_tracker: ComboTracker,
    lock_tracker: ComboTracker,
}

enum Decision {
    Allow,
    Consume,
}

impl GrabCtx {
    fn decide(&mut self, vk: u16, is_down: bool, is_e0: bool) -> Decision {
        let specific_vk = normalize_vk(vk, is_e0);
        let specific_key = specific_vk as u32;

        if is_down {
            self.mods.insert(specific_key);
        } else {
            self.mods.remove(&specific_key);
        }

        if !is_down {
            self.unlock_tracker.feed_key_release(specific_key);
            self.lock_tracker.feed_key_release(specific_key);
            return Decision::Allow;
        }

        let config = { self.state.read().config.clone() };

        let unlock_result = self.unlock_tracker.feed_key_press(specific_key);
        if unlock_result == ComboResult::Matched {
            self.unlock_tracker.reset();
            self.lock_tracker.reset();
            let was_locked = self.state.read().locked;
            if was_locked {
                self.state.write().locked = false;
                self.emit_lock_changed(false);
            }
            return Decision::Consume;
        }

        let lock_result = self.lock_tracker.feed_key_press(specific_key);
        if lock_result == ComboResult::Matched {
            self.unlock_tracker.reset();
            self.lock_tracker.reset();
            let was_locked = self.state.read().locked;
            if !was_locked {
                self.state.write().locked = true;
                self.emit_lock_changed(true);
            }
            return Decision::Consume;
        }

        let locked = self.state.read().locked;
        if !locked {
            self.state.write().total_allowed += 1;
            return Decision::Allow;
        }

        let should_block = config.block_all_devices;
        let _ = &self.block_map;

        if should_block {
            self.state.write().total_blocked += 1;
            return Decision::Consume;
        }

        let allow = filter::evaluate(
            &config,
            self.state.read().active_app.as_deref(),
            vk as u32,
            &self.mods,
        );
        if allow {
            self.state.write().total_allowed += 1;
            Decision::Allow
        } else {
            self.state.write().total_blocked += 1;
            Decision::Consume
        }
    }

    fn emit_lock_changed(&self, locked: bool) {
        if let Some(ref cb) = self.cb {
            cb("lock-state-changed", serde_json::json!({ "locked": locked }));
        }
    }
}

fn normalize_vk(vk: u16, is_e0: bool) -> u16 {
    match vk {
        VK_SHIFT => VK_SHIFT,
        VK_CONTROL if is_e0 => VK_RCONTROL,
        VK_CONTROL => VK_LCONTROL,
        VK_MENU if is_e0 => VK_RMENU,
        VK_MENU => VK_LMENU,
        VK_LWIN | VK_RWIN => VK_LWIN,
        _ => vk,
    }
}

static GRAB_CTX: OnceLock<Mutex<Option<GrabCtx>>> = OnceLock::new();

fn grab_ctx() -> &'static Mutex<Option<GrabCtx>> {
    GRAB_CTX.get_or_init(|| Mutex::new(None))
}

static HOOK_HANDLE: OnceLock<parking_lot::Mutex<Option<isize>>> = OnceLock::new();
fn hook_handle() -> &'static parking_lot::Mutex<Option<isize>> {
    HOOK_HANDLE.get_or_init(|| parking_lot::Mutex::new(None))
}

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: usize,
    l_param: isize,
) -> isize {
    use windows_sys::Win32::UI::WindowsAndMessaging::{CallNextHookEx, HC_ACTION};

    if n_code < 0 || n_code as u32 != HC_ACTION {
        return CallNextHookEx(null_mut(), n_code, w_param, l_param);
    }

    let msg = w_param as u32;
    let is_down = msg == 0x0100 || msg == 0x0104;
    let is_up = msg == 0x0101 || msg == 0x0105;
    if !is_down && !is_up {
        return CallNextHookEx(null_mut(), n_code, w_param, l_param);
    }

    let kb =
        &*(l_param as *const windows_sys::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT);
    let vk = kb.vkCode as u16;
    let flags = kb.flags;
    let is_e0 = flags & 0x01 != 0;

    let decision = {
        let Ok(mut guard) = grab_ctx().lock() else {
            return CallNextHookEx(null_mut(), n_code, w_param, l_param);
        };
        let Some(ref mut ctx) = *guard else {
            return CallNextHookEx(null_mut(), n_code, w_param, l_param);
        };
        ctx.decide(vk, is_down, is_e0)
    };

    match decision {
        Decision::Allow => CallNextHookEx(null_mut(), n_code, w_param, l_param),
        Decision::Consume => 1,
    }
}

pub fn run_grab_loop(
    state: Arc<RwLock<EngineState>>,
    running: Arc<AtomicBool>,
    event_cb: Option<EventCallback>,
    grab_thread_id: Arc<parking_lot::Mutex<Option<u32>>>,
) {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, MSG,
    };

    let config = { state.read().config.clone() };
    let devices = enumerate_keyboard_devices_with_config(&config.keyboard_devices);
    let mut block_map: HashMap<String, bool> = HashMap::new();
    for d in &devices {
        block_map.insert(d.instance_id.clone(), d.is_target);
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

    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            null_mut(),
            0,
        );
        if hook.is_null() {
            log::error!("Failed to install WH_KEYBOARD_LL hook");
            state.write().grab_active = false;
            return;
        }
        *hook_handle().lock() = Some(hook as isize);
    }

    grab_thread_id
        .lock()
        .clone_from(&Some(unsafe { GetCurrentThreadId() }));
    state.write().grab_active = true;
    log::info!("WH_KEYBOARD_LL hook installed");

    let mut msg: MSG = unsafe { std::mem::zeroed() };
    while running.load(Ordering::SeqCst) {
        let ret = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
        if ret == 0 || ret == -1 {
            break;
        }
    }

    unsafe {
        if let Some(h) = hook_handle().lock().take() {
            UnhookWindowsHookEx(h as *mut std::ffi::c_void);
        }
    }

    {
        let mut guard = grab_ctx().lock().unwrap();
        *guard = None;
    }

    log::info!("WH_KEYBOARD_LL grab loop stopped");
}
