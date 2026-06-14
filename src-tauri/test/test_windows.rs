use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

const VK_LCONTROL: u32 = 0xA2;
const VK_RCONTROL: u32 = 0xA3;
const VK_LMENU: u32 = 0xA4;
const VK_RMENU: u32 = 0xA5;
const VK_L: u32 = 0x4C;

static LOCKED: AtomicBool = AtomicBool::new(true);
static PRESSED_MODS: LazyLock<parking_lot::Mutex<HashSet<u32>>> =
    LazyLock::new(|| parking_lot::Mutex::new(HashSet::new()));

unsafe extern "system" fn keyboard_hook_proc(
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
        let mut mods = PRESSED_MODS.lock();
        match vk {
            VK_LCONTROL | VK_RCONTROL => {
                if is_down {
                    mods.insert(vk);
                } else {
                    mods.remove(&vk);
                }
            }
            VK_LMENU | VK_RMENU => {
                if is_down {
                    mods.insert(vk);
                } else {
                    mods.remove(&vk);
                }
            }
            _ => {}
        }
    }

    if is_down && LOCKED.load(Ordering::SeqCst) {
        let mods = PRESSED_MODS.lock();
        let ctrl = mods.contains(&VK_LCONTROL) || mods.contains(&VK_RCONTROL);
        let alt = mods.contains(&VK_LMENU) || mods.contains(&VK_RMENU);

        if ctrl && alt && vk == VK_L {
            LOCKED.store(false, Ordering::SeqCst);
            println!("\n[UNLOCKED] Keyboard unlocked! Press Ctrl+Alt+L to re-lock.");
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        let is_modifier =
            vk == VK_LCONTROL || vk == VK_RCONTROL || vk == VK_LMENU || vk == VK_RMENU;
        if is_modifier {
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        println!("[BLOCKED] Key vk=0x{:02X}", vk);
        return 1;
    }

    if is_down && !LOCKED.load(Ordering::SeqCst) {
        let mods = PRESSED_MODS.lock();
        let ctrl = mods.contains(&VK_LCONTROL) || mods.contains(&VK_RCONTROL);
        let alt = mods.contains(&VK_LMENU) || mods.contains(&VK_RMENU);

        if ctrl && alt && vk == VK_L {
            LOCKED.store(true, Ordering::SeqCst);
            println!("[LOCKED] Keyboard locked. Press Ctrl+Alt+L to unlock.");
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }
    }

    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

fn main() {
    println!("=== Keyboard Lock Test ===");
    println!("Installing WH_KEYBOARD_LL hook...");
    println!("Keyboard is now LOCKED. Most keys will be blocked.");
    println!("Press Ctrl+Alt+L to toggle lock/unlock.");
    println!("Press Ctrl+C in this console to quit.\n");

    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            std::ptr::null_mut(),
            0,
        );

        if hook.is_null() {
            eprintln!("FATAL: Failed to install WH_KEYBOARD_LL hook!");
            std::process::exit(1);
        }

        let tid = GetCurrentThreadId();
        println!("[OK] Hook installed. Thread ID: {}", tid);
        println!("[LOCKED] Keyboard is locked.\n");

        let mut msg: MSG = std::mem::zeroed();
        loop {
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 || ret == -1 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(hook);
        println!("Hook removed. Exiting.");
    }
}