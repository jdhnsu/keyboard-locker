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

    pub fn set_lightweight_mode(&self, enabled: bool) {
        self.state.write().lightweight_mode = enabled;
    }

    pub fn is_lightweight_mode(&self) -> bool {
        self.state.read().lightweight_mode
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

    pub fn restart_grab(&self) {
        self.stop_grab();
        self.start_grab();
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
        grab_loop_windows(state.clone(), running.clone(), event_cb, grab_thread_id.clone());
    }

    #[cfg(target_os = "linux")]
    {
        grab_loop_linux(state.clone(), running.clone(), event_cb, grab_thread_id.clone());
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (state, event_cb, grab_thread_id);
        log::info!("Keyboard grab loop started (no platform hook available)");

        while running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        log::info!("Keyboard grab loop stopped");
        return;
    }

    running.store(false, Ordering::SeqCst);
    state.write().grab_active = false;
    {
        let mut g = grab_thread_id.lock();
        *g = None;
    }
    log::info!("Keyboard grab loop exited");
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

#[cfg(target_os = "linux")]
mod linux_keymap {
    use std::collections::HashMap;

    pub fn evdev_to_vk(code: u16) -> Option<u32> {
        static MAP: std::sync::OnceLock<HashMap<u16, u32>> = std::sync::OnceLock::new();
        let map = MAP.get_or_init(|| {
            let mut m = HashMap::new();

            m.insert(1, 0x1B);   // KEY_ESC
            m.insert(2, 0x31);   // KEY_1
            m.insert(3, 0x32);   // KEY_2
            m.insert(4, 0x33);   // KEY_3
            m.insert(5, 0x34);   // KEY_4
            m.insert(6, 0x35);   // KEY_5
            m.insert(7, 0x36);   // KEY_6
            m.insert(8, 0x37);   // KEY_7
            m.insert(9, 0x38);   // KEY_8
            m.insert(10, 0x39);  // KEY_9
            m.insert(11, 0x30);  // KEY_0
            m.insert(12, 0xBD);  // KEY_MINUS
            m.insert(13, 0xBB);  // KEY_EQUAL
            m.insert(14, 0x08);  // KEY_BACKSPACE
            m.insert(15, 0x09);  // KEY_TAB
            m.insert(16, 0x51);  // KEY_Q
            m.insert(17, 0x57);  // KEY_W
            m.insert(18, 0x45);  // KEY_E
            m.insert(19, 0x52);  // KEY_R
            m.insert(20, 0x54);  // KEY_T
            m.insert(21, 0x59);  // KEY_Y
            m.insert(22, 0x55);  // KEY_U
            m.insert(23, 0x49);  // KEY_I
            m.insert(24, 0x4F);  // KEY_O
            m.insert(25, 0x50);  // KEY_P
            m.insert(26, 0xDB);  // KEY_LEFTBRACE
            m.insert(27, 0xDD);  // KEY_RIGHTBRACE
            m.insert(28, 0x0D);  // KEY_ENTER
            m.insert(29, 0xA2);  // KEY_LEFTCTRL
            m.insert(30, 0x41);  // KEY_A
            m.insert(31, 0x53);  // KEY_S
            m.insert(32, 0x44);  // KEY_D
            m.insert(33, 0x46);  // KEY_F
            m.insert(34, 0x47);  // KEY_G
            m.insert(35, 0x48);  // KEY_H
            m.insert(36, 0x4A);  // KEY_J
            m.insert(37, 0x4B);  // KEY_K
            m.insert(38, 0x4C);  // KEY_L
            m.insert(39, 0xBA);  // KEY_SEMICOLON
            m.insert(40, 0xDE);  // KEY_APOSTROPHE
            m.insert(41, 0xC0);  // KEY_GRAVE
            m.insert(42, 0xA0);  // KEY_LEFTSHIFT
            m.insert(43, 0xDC);  // KEY_BACKSLASH
            m.insert(44, 0x5A);  // KEY_Z
            m.insert(45, 0x58);  // KEY_X
            m.insert(46, 0x43);  // KEY_C
            m.insert(47, 0x56);  // KEY_V
            m.insert(48, 0x42);  // KEY_B
            m.insert(49, 0x4E);  // KEY_N
            m.insert(50, 0x4D);  // KEY_M
            m.insert(51, 0xBC);  // KEY_COMMA
            m.insert(52, 0xBE);  // KEY_DOT
            m.insert(53, 0xBF);  // KEY_SLASH
            m.insert(54, 0xA1);  // KEY_RIGHTSHIFT
            m.insert(55, 0x6A);  // KEY_KPASTERISK
            m.insert(56, 0xA4);  // KEY_LEFTALT
            m.insert(57, 0x20);  // KEY_SPACE
            m.insert(58, 0x14);  // KEY_CAPSLOCK
            m.insert(59, 0x70);  // KEY_F1
            m.insert(60, 0x71);  // KEY_F2
            m.insert(61, 0x72);  // KEY_F3
            m.insert(62, 0x73);  // KEY_F4
            m.insert(63, 0x74);  // KEY_F5
            m.insert(64, 0x75);  // KEY_F6
            m.insert(65, 0x76);  // KEY_F7
            m.insert(66, 0x77);  // KEY_F8
            m.insert(67, 0x78);  // KEY_F9
            m.insert(68, 0x79);  // KEY_F10
            m.insert(69, 0x90);  // KEY_NUMLOCK
            m.insert(70, 0x91);  // KEY_SCROLLLOCK
            m.insert(71, 0x67);  // KEY_KP7
            m.insert(72, 0x68);  // KEY_KP8
            m.insert(73, 0x69);  // KEY_KP9
            m.insert(74, 0x6B);  // KEY_KPMINUS
            m.insert(75, 0x64);  // KEY_KP4
            m.insert(76, 0x65);  // KEY_KP5
            m.insert(77, 0x66);  // KEY_KP6
            m.insert(78, 0x6D);  // KEY_KPPLUS
            m.insert(79, 0x61);  // KEY_KP1
            m.insert(80, 0x62);  // KEY_KP2
            m.insert(81, 0x63);  // KEY_KP3
            m.insert(82, 0x60);  // KEY_KP0
            m.insert(83, 0x6E);  // KEY_KPDOT
            m.insert(87, 0x7A);  // KEY_F11
            m.insert(88, 0x7B);  // KEY_F12
            m.insert(96, 0x0D);  // KEY_KPENTER
            m.insert(97, 0xA3);  // KEY_RIGHTCTRL
            m.insert(98, 0x6F);  // KEY_KPSLASH
            m.insert(99, 0x13);  // KEY_SYSRQ -> VK_PAUSE
            m.insert(100, 0xA5); // KEY_RIGHTALT
            m.insert(102, 0x24); // KEY_HOME
            m.insert(103, 0x26); // KEY_UP
            m.insert(104, 0x21); // KEY_PAGEUP
            m.insert(105, 0x25); // KEY_LEFT
            m.insert(106, 0x27); // KEY_RIGHT
            m.insert(107, 0x23); // KEY_END
            m.insert(108, 0x28); // KEY_DOWN
            m.insert(109, 0x22); // KEY_PAGEDOWN
            m.insert(110, 0x2D); // KEY_INSERT
            m.insert(111, 0x2E); // KEY_DELETE
            m.insert(125, 0x5B); // KEY_LEFTMETA
            m.insert(126, 0x5C); // KEY_RIGHTMETA

            m
        });
        map.get(&code).copied()
    }

    pub fn vk_to_evdev(vk: u32) -> Option<u16> {
        static MAP: std::sync::OnceLock<HashMap<u32, u16>> = std::sync::OnceLock::new();
        let map = MAP.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert(0x1B, 1);
            m.insert(0x31, 2);
            m.insert(0x32, 3);
            m.insert(0x33, 4);
            m.insert(0x34, 5);
            m.insert(0x35, 6);
            m.insert(0x36, 7);
            m.insert(0x37, 8);
            m.insert(0x38, 9);
            m.insert(0x39, 10);
            m.insert(0x30, 11);
            m.insert(0xBD, 12);
            m.insert(0xBB, 13);
            m.insert(0x08, 14);
            m.insert(0x09, 15);
            m.insert(0x51, 16);
            m.insert(0x57, 17);
            m.insert(0x45, 18);
            m.insert(0x52, 19);
            m.insert(0x54, 20);
            m.insert(0x59, 21);
            m.insert(0x55, 22);
            m.insert(0x49, 23);
            m.insert(0x4F, 24);
            m.insert(0x50, 25);
            m.insert(0xDB, 26);
            m.insert(0xDD, 27);
            m.insert(0x0D, 28);
            m.insert(0xA2, 29);
            m.insert(0x41, 30);
            m.insert(0x53, 31);
            m.insert(0x44, 32);
            m.insert(0x46, 33);
            m.insert(0x47, 34);
            m.insert(0x48, 35);
            m.insert(0x4A, 36);
            m.insert(0x4B, 37);
            m.insert(0x4C, 38);
            m.insert(0xBA, 39);
            m.insert(0xDE, 40);
            m.insert(0xC0, 41);
            m.insert(0xA0, 42);
            m.insert(0xDC, 43);
            m.insert(0x5A, 44);
            m.insert(0x58, 45);
            m.insert(0x43, 46);
            m.insert(0x56, 47);
            m.insert(0x42, 48);
            m.insert(0x4E, 49);
            m.insert(0x4D, 50);
            m.insert(0xBC, 51);
            m.insert(0xBE, 52);
            m.insert(0xBF, 53);
            m.insert(0xA1, 54);
            m.insert(0x6A, 55);
            m.insert(0xA4, 56);
            m.insert(0x20, 57);
            m.insert(0x14, 58);
            m.insert(0x70, 59);
            m.insert(0x71, 60);
            m.insert(0x72, 61);
            m.insert(0x73, 62);
            m.insert(0x74, 63);
            m.insert(0x75, 64);
            m.insert(0x76, 65);
            m.insert(0x77, 66);
            m.insert(0x78, 67);
            m.insert(0x79, 68);
            m.insert(0x7A, 87);
            m.insert(0x7B, 88);
            m.insert(0xA3, 97);
            m.insert(0x6F, 98);
            m.insert(0xA5, 100);
            m.insert(0x24, 102);
            m.insert(0x26, 103);
            m.insert(0x21, 104);
            m.insert(0x25, 105);
            m.insert(0x27, 106);
            m.insert(0x23, 107);
            m.insert(0x28, 108);
            m.insert(0x22, 109);
            m.insert(0x2D, 110);
            m.insert(0x2E, 111);
            m.insert(0x5B, 125);
            m.insert(0x5C, 126);
            m.insert(0x60, 82);
            m.insert(0x61, 79);
            m.insert(0x62, 80);
            m.insert(0x63, 81);
            m.insert(0x64, 75);
            m.insert(0x65, 76);
            m.insert(0x66, 77);
            m.insert(0x67, 71);
            m.insert(0x68, 72);
            m.insert(0x69, 73);
            m.insert(0x6B, 74);
            m.insert(0x6D, 78);
            m.insert(0x6E, 83);
            m.insert(0x90, 69);
            m.insert(0x91, 70);
            m.insert(0x13, 99);
            m
        });
        map.get(&vk).copied()
    }

    #[cfg(target_os = "windows")]
    pub fn is_modifier_vk(vk: u32) -> bool {
        matches!(
            vk,
            0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0x5B | 0x5C
        )
    }

    #[cfg(target_os = "linux")]
    pub fn is_combo_modifier_vk(vk: u32) -> bool {
        matches!(vk, 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5)
    }
}

#[cfg(target_os = "linux")]
fn grab_loop_linux(
    state: Arc<RwLock<EngineState>>,
    running: Arc<AtomicBool>,
    event_cb: Option<EventCallback>,
    grab_thread_id: Arc<parking_lot::Mutex<Option<u32>>>,
) {
    use evdev::{uinput::VirtualDeviceBuilder, Device, EventType, InputEvent, Key};
    use std::os::unix::io::AsRawFd;

    let mut keyboard_devices: Vec<Device> = Vec::new();

    for (_path, device) in evdev::enumerate() {
        let has_keys = device.supported_keys().map_or(false, |keys| {
            keys.contains(Key::KEY_A) || keys.contains(Key::KEY_1)
        });
        if has_keys {
            log::info!(
                "Found keyboard device: {} at {:?}",
                device.name().unwrap_or("?"),
                _path
            );
            keyboard_devices.push(device);
        }
    }

    if keyboard_devices.is_empty() {
        log::error!("No keyboard devices found via evdev");
        if let Some(ref cb) = event_cb {
            cb("grab-error", serde_json::json!({"error": "No keyboard devices found via evdev", "detail": "No /dev/input/event* devices with keyboard keys detected"}));
        }
        return;
    }

    let mut all_supported_keys = evdev::AttributeSet::<Key>::new();
    for device in &keyboard_devices {
        if let Some(keys) = device.supported_keys() {
            for key in keys.iter() {
                all_supported_keys.insert(key);
            }
        }
    }

    let virtual_dev = match VirtualDeviceBuilder::new() {
        Ok(builder) => builder,
        Err(e) => {
            log::error!("Failed to open /dev/uinput: {}", e);
            if let Some(ref cb) = event_cb {
                cb("grab-error", serde_json::json!({"error": format!("Failed to open /dev/uinput: {}", e), "detail": "Ensure uinput module is loaded and you have write permission"}));
            }
            return;
        }
    };

    let virtual_dev = match virtual_dev
        .name("KeyLock Pro Virtual Keyboard")
        .with_keys(&all_supported_keys)
    {
        Ok(builder) => match builder.build() {
            Ok(d) => {
                log::info!("Created uinput virtual device for key injection");
                d
            }
            Err(e) => {
                log::error!("Failed to build uinput device: {}", e);
                if let Some(ref cb) = event_cb {
                    cb("grab-error", serde_json::json!({"error": format!("Failed to build uinput device: {}", e)}));
                }
                return;
            }
        },
        Err(e) => {
            log::error!("Failed to configure uinput device: {}", e);
            if let Some(ref cb) = event_cb {
                cb("grab-error", serde_json::json!({"error": format!("Failed to configure uinput device: {}", e)}));
            }
            return;
        }
    };
    let uinput = Arc::new(parking_lot::Mutex::new(virtual_dev));

    let mut grabbed_count = 0usize;
    let mut grab_errors: Vec<String> = Vec::new();
    for device in &mut keyboard_devices {
        if let Err(e) = device.grab() {
            log::error!("Failed to grab device '{}': {}", device.name().unwrap_or("?"), e);
            grab_errors.push(format!("{}: {}", device.name().unwrap_or("?"), e));
        } else {
            log::info!("Grabbed device: {}", device.name().unwrap_or("?"));
            grabbed_count += 1;
        }
    }

    if grabbed_count == 0 && !keyboard_devices.is_empty() {
        if let Some(ref cb) = event_cb {
            cb("grab-warning", serde_json::json!({
                "warning": "No devices could be exclusively grabbed",
                "detail": "The display server may already have exclusive access. Keyboard interception may not work.",
                "errors": grab_errors
            }));
        }
    }

    {
        let tid = std::process::id();
        let mut g = grab_thread_id.lock();
        *g = Some(tid);
    }

    let pressed_modifiers: Arc<parking_lot::Mutex<HashSet<u32>>> =
        Arc::new(parking_lot::Mutex::new(HashSet::new()));

    log::info!(
        "Linux evdev grab loop started, monitoring {} device(s)",
        keyboard_devices.len()
    );

    let mut poll_fds: Vec<libc::pollfd> = keyboard_devices
        .iter()
        .map(|d| libc::pollfd {
            fd: d.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        })
        .collect();

    let mut pending_events: Vec<Vec<InputEvent>> = vec![Vec::new(); keyboard_devices.len()];

    while running.load(Ordering::SeqCst) {
        let ret = unsafe {
            libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as libc::nfds_t, 50)
        };

        if ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() != std::io::ErrorKind::Interrupted {
                log::error!("poll() failed: {}", err);
            }
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        for (i, device) in keyboard_devices.iter_mut().enumerate() {
            if poll_fds[i].revents & libc::POLLIN == 0 {
                continue;
            }
            poll_fds[i].revents = 0;

            let device_name = device.name().unwrap_or("?").to_string();
            match device.fetch_events() {
                Ok(iter) => {
                    pending_events[i].extend(iter);
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        log::warn!("Error reading events from '{}': {}", device_name, e);
                    }
                }
            }
        }

        for (i, _device) in keyboard_devices.iter_mut().enumerate() {
            let events: Vec<InputEvent> = std::mem::take(&mut pending_events[i]);
            if events.is_empty() {
                continue;
            }

            let mut batch_to_emit: Vec<InputEvent> = Vec::with_capacity(events.len());

            for ev in events {
                if ev.event_type() != EventType::KEY {
                    batch_to_emit.push(ev);
                    continue;
                }

                let code = ev.code();
                let value = ev.value();
                let is_down = value == 1;
                let is_up = value == 0;
                let is_repeat = value == 2;

                let Some(vk) = linux_keymap::evdev_to_vk(code) else {
                    batch_to_emit.push(ev);
                    continue;
                };

                if is_down || is_up {
                    let mut mods = pressed_modifiers.lock();
                    match vk {
                        0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0x5B | 0x5C => {
                            if is_down {
                                mods.insert(vk);
                            } else {
                                mods.remove(&vk);
                            }
                        }
                        _ => {}
                    }
                }

                if is_repeat {
                    let s = state.read();
                    if s.locked {
                        let mods = pressed_modifiers.lock();
                        let allow = filter::evaluate(
                            &s.config,
                            s.active_app.as_deref(),
                            vk,
                            &mods,
                        );
                        if allow {
                            batch_to_emit.push(ev);
                        }
                    } else {
                        batch_to_emit.push(ev);
                    }
                    continue;
                }

                let is_shortcut = {
                    let s = state.read();
                    let unlock = &s.config.unlock_combo;
                    let lock = &s.config.lock_combo;
                    unlock.contains(&vk)
                        || lock.contains(&vk)
                        || linux_keymap::is_combo_modifier_vk(vk)
                };

                if is_shortcut {
                    let combo_result = {
                        let mut s = state.write();
                        if is_down {
                            s.combo_tracker.feed_key_press(vk)
                        } else {
                            s.combo_tracker.feed_key_release(vk);
                            ComboResult::InProgress
                        }
                    };

                    if combo_result == ComboResult::Matched {
                        let was_locked = state.read().locked;
                        if was_locked {
                            let mut s = state.write();
                            s.locked = false;
                            s.combo_tracker.reset();
                            s.lock_combo_tracker.reset();
                            drop(s);
                            if let Some(ref cb) = event_cb {
                                cb("lock-state-changed", serde_json::json!({"locked": false}));
                            }
                        } else {
                            let mut s = state.write();
                            s.locked = true;
                            s.combo_tracker.reset();
                            s.lock_combo_tracker.reset();
                            drop(s);
                            if let Some(ref cb) = event_cb {
                                cb("lock-state-changed", serde_json::json!({"locked": true}));
                            }
                        }
                    }

                    batch_to_emit.push(ev);
                    continue;
                }

                let s = state.read();
                if !s.locked {
                    drop(s);
                    batch_to_emit.push(ev);
                    continue;
                }

                let mods = pressed_modifiers.lock();
                let allow = filter::evaluate(&s.config, s.active_app.as_deref(), vk, &mods);
                drop(mods);

                if allow {
                    drop(s);
                    let mut s = state.write();
                    s.total_allowed += 1;
                    drop(s);
                    batch_to_emit.push(ev);
                } else {
                    drop(s);
                    let mut s = state.write();
                    s.total_blocked += 1;
                    drop(s);
                }
            }

            if !batch_to_emit.is_empty() {
                let mut uinput_guard = uinput.lock();
                if let Err(e) = uinput_guard.emit(&batch_to_emit) {
                    log::warn!("Failed to emit events to uinput: {}", e);
                }
            }
        }
    }

    for device in &mut keyboard_devices {
        let _ = device.ungrab();
    }
    log::info!("Linux evdev grab loop stopped, devices ungrabbed");
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
        let poll_interval = {
            let s = state.read();
            if s.lightweight_mode {
                Duration::from_secs(2)
            } else {
                Duration::from_millis(500)
            }
        };
        thread::sleep(poll_interval);
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