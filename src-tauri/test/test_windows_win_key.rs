//! Dedicated test for Win key (VK_LWIN=0x5B, VK_RWIN=0x5C) blocking behavior.
//!
//! Background:
//!   The main keyboard hook tracks Ctrl (0xA2/0xA3) and Alt (0xA4/0xA5) as modifiers
//!   for the Ctrl+Alt+L toggle.  Win keys (0x5B/0x5C) are NOT tracked — they fall through
//!   to the default "return 1" block path when locked.
//!
//!   However, Windows 10+ has special handling for the Win key at a privileged level
//!   (to prevent malware from trapping the Start Menu).  This means WH_KEYBOARD_LL
//!   hooks may be IGNORED for Win key events, causing the Start Menu or Win+Key
//!   shortcuts (Win+E, Win+R, Win+D, etc.) to still fire even while "locked".
//!
//!   Additionally, not tracking Win in PRESSED_MODS means if Win is physically held
//!   and another key is pressed, the hook only blocks the second key but the system
//!   sees Win+Key as a shortcut — bypassing the lock entirely.
//!
//! Run:  cd src-tauri\test && cargo run
//!   or: cd src-tauri\test && cargo run --example test_windows_win_key  (if added as [[example]])

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

// Known virtual-key codes
const VK_LCONTROL: u32 = 0xA2;
const VK_RCONTROL: u32 = 0xA3;
const VK_LMENU: u32 = 0xA4;
const VK_RMENU: u32 = 0xA5;
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;
const VK_L: u32 = 0x4C;

static LOCKED: AtomicBool = AtomicBool::new(true);
static PRESSED_MODS: LazyLock<parking_lot::Mutex<HashSet<u32>>> =
    LazyLock::new(|| parking_lot::Mutex::new(HashSet::new()));

// Counters for diagnostics
static WIN_DOWN_BLOCKED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static WIN_UP_BLOCKED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn vk_name(vk: u32) -> &'static str {
    match vk {
        VK_LCONTROL => "LeftCtrl",
        VK_RCONTROL => "RightCtrl",
        VK_LMENU => "LeftAlt",
        VK_RMENU => "RightAlt",
        VK_LWIN => "LeftWin",
        VK_RWIN => "RightWin",
        VK_L => "L",
        _ => "(other)",
    }
}

fn is_win_key(vk: u32) -> bool {
    vk == VK_LWIN || vk == VK_RWIN
}

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

    // ── Track all modifier keys (including Win) ──────────────────────────
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
            VK_LWIN | VK_RWIN => {
                if is_down {
                    mods.insert(vk);
                } else {
                    mods.remove(&vk);
                }
            }
            _ => {}
        }
    }

    // ── Toggle: Ctrl+Alt+L ───────────────────────────────────────────────
    if is_down && LOCKED.load(Ordering::SeqCst) {
        let mods = PRESSED_MODS.lock();
        let ctrl = mods.contains(&VK_LCONTROL) || mods.contains(&VK_RCONTROL);
        let alt = mods.contains(&VK_LMENU) || mods.contains(&VK_RMENU);

        if ctrl && alt && vk == VK_L {
            LOCKED.store(false, Ordering::SeqCst);
            println!("\n[UNLOCKED] Keyboard unlocked! Press Ctrl+Alt+L to re-lock.");
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }
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

    // ── Blocking logic ───────────────────────────────────────────────────
    if LOCKED.load(Ordering::SeqCst) {
        let is_modifier = vk == VK_LCONTROL
            || vk == VK_RCONTROL
            || vk == VK_LMENU
            || vk == VK_RMENU
            || vk == VK_LWIN   // allow Win to pass through so we can track it
            || vk == VK_RWIN;

        if is_modifier {
            // Modifiers must pass through so system tracks their physical state.
            // This is needed for the Ctrl+Alt+L toggle and to avoid Win key
            // getting "stuck" in a phantom pressed state.
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        // ── Check if Win key is held: Win+Key shortcuts must be blocked ──
        if is_down {
            let mods = PRESSED_MODS.lock();
            let win_held = mods.contains(&VK_LWIN) || mods.contains(&VK_RWIN);

            if win_held {
                // Win+Key shortcut — block it explicitly
                println!(
                    "[BLOCKED-WinCombo] Win+{}  vk=0x{:02X}",
                    vk_name(vk),
                    vk
                );
                return 1;
            }

            if is_win_key(vk) {
                WIN_DOWN_BLOCKED.fetch_add(1, Ordering::Relaxed);
            }
            println!("[BLOCKED] Key {}  vk=0x{:02X}", vk_name(vk), vk);
            return 1;
        }

        // Key-up for non-modifier while locked — also block to prevent leaks
        if is_up {
            if is_win_key(vk) {
                WIN_UP_BLOCKED.fetch_add(1, Ordering::Relaxed);
            }
            return 1;
        }
    }

    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║     Win Key Isolation Test — WH_KEYBOARD_LL hook        ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  GOAL: Verify whether Win key can be blocked by a       ║");
    println!("║  low-level keyboard hook on this Windows version.       ║");
    println!("║                                                        ║");
    println!("║  This test HOOK TRACKS Win (0x5B/0x5C) as a modifier    ║");
    println!("║  and lets it pass through so its up/down state is       ║");
    println!("║  known. Win+Key combos are explicitly blocked.          ║");
    println!("║                                                        ║");
    println!("║  ─── Test steps ────────────────────────────────────    ║");
    println!("║  1. Observe: Start Menu should NOT open on Win press    ║");
    println!("║  2. Try Win+E — Explorer should NOT open                ║");
    println!("║  3. Try Win+R — Run dialog should NOT open              ║");
    println!("║  4. Try Win+D — Desktop should NOT toggle               ║");
    println!("║  5. Press Ctrl+Alt+L to toggle lock/unlock              ║");
    println!("║  6. Press Ctrl+C in this console to quit                ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            std::ptr::null_mut(),
            0,
        );

        if hook.is_null() {
            let err = std::io::Error::last_os_error();
            eprintln!("FATAL: Failed to install WH_KEYBOARD_LL hook!  Error: {err}");
            std::process::exit(1);
        }

        let tid = GetCurrentThreadId();
        println!("[OK] Hook installed on thread {}. Keyboard is LOCKED.\n", tid);

        // ── Message pump ──────────────────────────────────────────────
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

        // ── Print diagnostics ─────────────────────────────────────────
        let down = WIN_DOWN_BLOCKED.load(Ordering::Relaxed);
        let up = WIN_UP_BLOCKED.load(Ordering::Relaxed);
        println!();
        println!("=== DIAGNOSTICS ===");
        println!("Win KEYDOWN events blocked: {}", down);
        println!("Win KEYUP   events blocked: {}", up);
        if down > 0 && up == 0 {
            println!(
                "WARNING: Win down events were seen but NO up events — this can "
            );
            println!(
                "         leave Win in a 'stuck' phantom state, causing next "
            );
            println!("         keypress to register as Win+Key!");
        }
        println!("Hook removed. Exiting.");
    }
}
