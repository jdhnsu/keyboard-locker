use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::combo::{ComboResult, ComboTracker};
use super::filter;
use super::rules::Config;
use crate::platform::PlatformExtras;
use crate::state::EngineState;

#[cfg(target_os = "windows")]
static SHORTCUT_KEYS: parking_lot::Mutex<Option<Vec<u32>>> = parking_lot::const_mutex(None);

#[cfg(target_os = "windows")]
const VK_SHIFT: u32 = 16;
#[cfg(target_os = "windows")]
const VK_CONTROL: u32 = 17;
#[cfg(target_os = "windows")]
const VK_MENU: u32 = 18;
#[cfg(target_os = "windows")]
const VK_LSHIFT: u32 = 160;
#[cfg(target_os = "windows")]
const VK_RSHIFT: u32 = 161;
#[cfg(target_os = "windows")]
const VK_LCONTROL: u32 = 162;
#[cfg(target_os = "windows")]
const VK_RCONTROL: u32 = 163;
#[cfg(target_os = "windows")]
const VK_LMENU: u32 = 164;
#[cfg(target_os = "windows")]
const VK_RMENU: u32 = 165;

#[cfg(target_os = "windows")]
fn normalize_modifier(vk: u32) -> Option<u32> {
    match vk {
        VK_LSHIFT | VK_RSHIFT | VK_SHIFT => Some(VK_SHIFT),
        VK_LCONTROL | VK_RCONTROL | VK_CONTROL => Some(VK_CONTROL),
        VK_LMENU | VK_RMENU | VK_MENU => Some(VK_MENU),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn is_shortcut_key(vk: u32) -> bool {
    let guard = SHORTCUT_KEYS.lock();
    if let Some(ref keys) = *guard {
        if keys.contains(&vk) {
            return true;
        }
        if let Some(norm) = normalize_modifier(vk) {
            if keys.contains(&norm) {
                return true;
            }
        }
        for &combo_vk in keys {
            if let Some(norm) = normalize_modifier(combo_vk) {
                if norm == normalize_modifier(vk).unwrap_or(vk) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn update_hook_shortcut_keys(unlock_combo: &[u32], lock_combo: &[u32]) {
    #[cfg(target_os = "windows")]
    {
        let mut keys: Vec<u32> = Vec::new();
        for &vk in unlock_combo {
            if !keys.contains(&vk) {
                keys.push(vk);
            }
        }
        for &vk in lock_combo {
            if !keys.contains(&vk) {
                keys.push(vk);
            }
        }
        let mut guard = SHORTCUT_KEYS.lock();
        *guard = Some(keys);
    }
    let _ = (unlock_combo, lock_combo);
}

pub type EventCallback = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync + 'static>;

pub struct Engine {
    pub state: Arc<RwLock<EngineState>>,
    running: Arc<AtomicBool>,
    grab_thread_id: Arc<parking_lot::Mutex<Option<u32>>>,
    event_cb: RwLock<Option<EventCallback>>,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        let state = EngineState::new(config);
        Engine {
            state: Arc::new(RwLock::new(state)),
            running: Arc::new(AtomicBool::new(false)),
            grab_thread_id: Arc::new(parking_lot::Mutex::new(None)),
            event_cb: RwLock::new(None),
        }
    }

    pub fn set_event_callback(&self, cb: EventCallback) {
        *self.event_cb.write() = Some(cb);
    }

    fn emit(&self, event: &str, payload: serde_json::Value) {
        if let Some(ref cb) = *self.event_cb.read() {
            cb(event, payload);
        }
    }

    pub fn handle_key_press(&self, key_code: u32, modifiers: &HashSet<u32>) -> Option<u32> {
        let mut state = self.state.write();

        let combo_result = state.combo_tracker.feed_key_press(key_code);
        if combo_result == ComboResult::Matched {
            let was_locked = state.locked;
            state.locked = false;
            drop(state);
            if was_locked {
                self.emit_lock_changed(false);
            }
            return None;
        }

        if !state.locked {
            state.total_allowed += 1;
            return Some(key_code);
        }

        let allow = filter::evaluate(
            &state.config,
            state.active_app.as_deref(),
            key_code,
            modifiers,
        );
        if allow {
            state.total_allowed += 1;
            Some(key_code)
        } else {
            state.total_blocked += 1;
            None
        }
    }

    pub fn handle_key_release(&self, key_code: u32) {
        let mut state = self.state.write();
        state.combo_tracker.feed_key_release(key_code);
    }

    pub fn lock(&self) {
        let prev = {
            let mut state = self.state.write();
            let prev = state.locked;
            state.locked = true;
            state.combo_tracker.reset();
            prev
        };
        if !prev {
            self.emit_lock_changed(true);
            self.start_auto_unlock_timer();
        }
    }

    pub fn unlock(&self) {
        let prev = {
            let mut state = self.state.write();
            let prev = state.locked;
            state.locked = false;
            state.combo_tracker.reset();
            prev
        };
        if prev {
            self.emit_lock_changed(false);
        }
    }

    pub fn toggle(&self) -> bool {
        let (prev, now) = {
            let mut state = self.state.write();
            let prev = state.locked;
            state.locked = !state.locked;
            if state.locked {
                state.combo_tracker.reset();
            }
            (prev, state.locked)
        };
        if prev != now {
            self.emit_lock_changed(now);
            if now {
                self.start_auto_unlock_timer();
            }
        }
        now
    }

    pub fn is_locked(&self) -> bool {
        self.state.read().locked
    }

    pub fn set_active_app(&self, app_name: Option<String>) {
        self.state.write().active_app = app_name;
    }

    pub fn set_grab_active(&self, active: bool) {
        self.state.write().grab_active = active;
    }

    pub fn update_config(&self, config: Config) {
        let mut state = self.state.write();
        let unlock_sequence = config.unlock_combo.clone();
        let lock_sequence = config.lock_combo.clone();
        state.config = config;
        state.combo_tracker = ComboTracker::new(unlock_sequence);
        state.lock_combo_tracker = ComboTracker::new(lock_sequence);
    }

    pub fn get_snapshot(&self) -> EngineSnapshot {
        let state = self.state.read();
        EngineSnapshot {
            locked: state.locked,
            grab_active: state.grab_active,
            total_blocked: state.total_blocked,
            total_allowed: state.total_allowed,
            active_app: state.active_app.clone(),
            combo_progress: state.combo_tracker.progress(),
            lock_combo_progress: state.lock_combo_tracker.progress(),
        }
    }

    pub fn start_grab(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        self.state.write().grab_active = true;

        let state = self.state.clone();
        let running = self.running.clone();
        let event_cb = self.event_cb.read().clone();
        let grab_thread_id = self.grab_thread_id.clone();

        thread::Builder::new()
            .name("keylock-grab".into())
            .spawn(move || {
                grab_loop_impl(state, running, event_cb, grab_thread_id);
            })
            .expect("failed to spawn grab thread");
    }

    pub fn stop_grab(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.state.write().grab_active = false;

        #[cfg(target_os = "windows")]
        {
            let tid = self.grab_thread_id.lock();
            if let Some(tid) = *tid {
                unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                        tid,
                        windows_sys::Win32::UI::WindowsAndMessaging::WM_QUIT,
                        0,
                        0,
                    );
                }
            }
        }
    }

    pub fn start_foreground_tracker(&self) {
        let state = self.state.clone();
        let running = self.running.clone();

        thread::Builder::new()
            .name("keylock-fg-tracker".into())
            .spawn(move || {
                foreground_tracker_loop(state, running);
            })
            .expect("failed to spawn foreground tracker thread");
    }

    fn start_auto_unlock_timer(&self) {
        let timeout = {
            let s = self.state.read();
            s.config.auto_unlock_timeout
        };

        if let Some(secs) = timeout {
            let state = self.state.clone();
            let cb = self.event_cb.read().clone();

            thread::Builder::new()
                .name("keylock-auto-unlock".into())
                .spawn(move || {
                    let deadline = std::time::Instant::now() + Duration::from_secs(secs);
                    loop {
                        let now = std::time::Instant::now();
                        if now >= deadline {
                            let mut s = state.write();
                            if s.locked {
                                s.locked = false;
                                drop(s);
                                if let Some(ref c) = cb {
                                    c("lock-state-changed", serde_json::json!({"locked": false}));
                                }
                            }
                            break;
                        }
                        let remaining = deadline - now;
                        let sleep_ms = remaining.as_millis().min(1000) as u64;
                        thread::sleep(Duration::from_millis(sleep_ms));

                        if !state.read().locked {
                            break;
                        }
                    }
                })
                .expect("failed to spawn auto-unlock thread");
        }
    }

    fn emit_lock_changed(&self, locked: bool) {
        self.emit(
            "lock-state-changed",
            serde_json::json!({ "locked": locked }),
        );
    }
}

fn grab_loop_impl(
    state: Arc<RwLock<EngineState>>,
    running: Arc<AtomicBool>,
    event_cb: Option<EventCallback>,
    grab_thread_id: Arc<parking_lot::Mutex<Option<u32>>>,
) {
    #[cfg(target_os = "windows")]
    {
        grab_loop_windows(state, running, event_cb, grab_thread_id);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (state, event_cb, grab_thread_id);
        log::info!("Keyboard grab loop started (no platform hook available)");

        while running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        log::info!("Keyboard grab loop stopped");
    }
}

#[cfg(target_os = "windows")]
fn grab_loop_windows(
    state: Arc<RwLock<EngineState>>,
    running: Arc<AtomicBool>,
    event_cb: Option<EventCallback>,
    grab_thread_id: Arc<parking_lot::Mutex<Option<u32>>>,
) {
    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, MSG,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    static GLOBAL_STATE: parking_lot::Mutex<Option<Arc<RwLock<EngineState>>>> =
        parking_lot::const_mutex(None);
    static GLOBAL_EVENT_CB: parking_lot::Mutex<Option<EventCallback>> =
        parking_lot::const_mutex(None);
    static GLOBAL_HOOK: parking_lot::Mutex<Option<usize>> = parking_lot::const_mutex(None);
    static PRESSED_MODIFIERS: parking_lot::Mutex<Option<HashSet<u32>>> =
        parking_lot::const_mutex(None);

    unsafe extern "system" fn low_level_keyboard_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code as u32 != HC_ACTION {
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        let kb = &*(l_param as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode as u32;
        let is_down = w_param == WM_KEYDOWN as usize || w_param == WM_SYSKEYDOWN as usize;
        let is_up = w_param == WM_KEYUP as usize || w_param == WM_SYSKEYUP as usize;

        if is_down || is_up {
            {
                let mut guard = PRESSED_MODIFIERS.lock();
                if let Some(ref mut mods) = *guard {
                    match vk {
                        VK_LSHIFT | VK_RSHIFT | VK_SHIFT => {
                            if is_down {
                                mods.insert(vk);
                            } else {
                                mods.remove(&vk);
                            }
                        }
                        VK_LCONTROL | VK_RCONTROL | VK_CONTROL => {
                            if is_down {
                                mods.insert(vk);
                            } else {
                                mods.remove(&vk);
                            }
                        }
                        VK_LMENU | VK_RMENU | VK_MENU => {
                            if is_down {
                                mods.insert(vk);
                            } else {
                                mods.remove(&vk);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if is_down || is_up {
            if is_shortcut_key(vk) {
                return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
            }
        }

        if is_down {
            let state_arc = {
                let state_opt = GLOBAL_STATE.lock();
                state_opt.clone()
            };
            if let Some(st) = state_arc {
                let mut s = st.write();

                if s.locked {
                    let modifiers_guard = PRESSED_MODIFIERS.lock();
                    let modifiers = modifiers_guard.as_ref().unwrap_or_else(|| unreachable!());
                    let allow = filter::evaluate(&s.config, s.active_app.as_deref(), vk, modifiers);
                    if allow {
                        s.total_allowed += 1;
                        drop(s);
                        CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
                    } else {
                        s.total_blocked += 1;
                        1
                    }
                } else {
                    s.total_allowed += 1;
                    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
                }
            } else {
                CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
            }
        } else if is_up {
            CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
        } else {
            CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
        }
    }

    log::info!("Keyboard grab loop starting on Windows with WH_KEYBOARD_LL");

    {
        let mut g = GLOBAL_STATE.lock();
        *g = Some(state.clone());
    }
    {
        let mut g = GLOBAL_EVENT_CB.lock();
        *g = event_cb;
    }
    {
        let mut g = PRESSED_MODIFIERS.lock();
        *g = Some(HashSet::new());
    }
    {
        let s = state.read();
        update_hook_shortcut_keys(&s.config.unlock_combo, &s.config.lock_combo);
    }

    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            std::ptr::null_mut(),
            0,
        );

        if hook.is_null() {
            log::error!("Failed to install WH_KEYBOARD_LL hook");
            return;
        }

        {
            let mut g = GLOBAL_HOOK.lock();
            *g = Some(hook as usize);
        }

        {
            let tid = GetCurrentThreadId();
            let mut g = grab_thread_id.lock();
            *g = Some(tid);
        }

        log::info!("WH_KEYBOARD_LL hook installed successfully");

        let mut msg: MSG = std::mem::zeroed();
        while running.load(Ordering::SeqCst) {
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 || ret == -1 {
                break;
            }
        }

        {
            let mut g = GLOBAL_HOOK.lock();
            if let Some(h) = g.take() {
                UnhookWindowsHookEx(h as *mut std::ffi::c_void);
            }
        }

        {
            let mut g = GLOBAL_STATE.lock();
            *g = None;
        }

        log::info!("WH_KEYBOARD_LL hook removed, grab loop stopped");
    }
}

fn foreground_tracker_loop(state: Arc<RwLock<EngineState>>, running: Arc<AtomicBool>) {
    log::info!("Foreground process tracker started");

    let platform = crate::platform::create_platform();

    while running.load(Ordering::SeqCst) {
        let app = platform.get_foreground_process();
        {
            let mut s = state.write();
            s.active_app = app;
        }
        thread::sleep(Duration::from_millis(500));
    }

    log::info!("Foreground process tracker stopped");
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineSnapshot {
    pub locked: bool,
    pub grab_active: bool,
    pub total_blocked: u64,
    pub total_allowed: u64,
    pub active_app: Option<String>,
    pub combo_progress: (usize, usize),
    pub lock_combo_progress: (usize, usize),
}