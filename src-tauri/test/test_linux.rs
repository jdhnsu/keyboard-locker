use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use evdev::{uinput::VirtualDeviceBuilder, Device, EventType, InputEvent, Key};

const KEY_LEFTCTRL: u16 = 29;
const KEY_RIGHTCTRL: u16 = 97;
const KEY_LEFTALT: u16 = 56;
const KEY_RIGHTALT: u16 = 100;
const KEY_L: u16 = 38;

static LOCKED: AtomicBool = AtomicBool::new(true);

fn key_name(code: u16) -> &'static str {
    match code {
        KEY_LEFTCTRL => "LeftCtrl",
        KEY_RIGHTCTRL => "RightCtrl",
        KEY_LEFTALT => "LeftAlt",
        KEY_RIGHTALT => "RightAlt",
        KEY_L => "L",
        _ => "(other)",
    }
}

fn is_ctrl(code: u16) -> bool {
    code == KEY_LEFTCTRL || code == KEY_RIGHTCTRL
}

fn is_alt(code: u16) -> bool {
    code == KEY_LEFTALT || code == KEY_RIGHTALT
}

fn is_modifier(code: u16) -> bool {
    is_ctrl(code) || is_alt(code)
}

fn main() {
    println!("=== Keyboard Lock Test (Linux evdev) ===");
    println!("Enumerating evdev keyboard devices...");

    let mut keyboard_devices: Vec<Device> = Vec::new();
    for (_path, device) in evdev::enumerate() {
        let has_keys = device.supported_keys().map_or(false, |keys| {
            keys.contains(Key::KEY_A) || keys.contains(Key::KEY_1)
        });
        if has_keys {
            println!(
                "  Found keyboard device: {} at {:?}",
                device.name().unwrap_or("?"),
                _path
            );
            keyboard_devices.push(device);
        }
    }

    if keyboard_devices.is_empty() {
        eprintln!("FATAL: No keyboard devices found via evdev!");
        eprintln!("Make sure you have read access to /dev/input/event* devices.");
        std::process::exit(1);
    }

    let supported_keys = match keyboard_devices[0].supported_keys() {
        Some(keys) => keys,
        None => {
            eprintln!("FATAL: No supported keys found on device!");
            std::process::exit(1);
        }
    };

    let mut virtual_dev = match VirtualDeviceBuilder::new() {
        Ok(builder) => match builder
            .name("Test Keyboard Lock Virtual Device")
            .with_keys(supported_keys)
        {
            Ok(builder) => match builder.build() {
                Ok(d) => {
                    println!("[OK] Created uinput virtual device for key injection");
                    d
                }
                Err(e) => {
                    eprintln!("FATAL: Failed to build uinput device: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("FATAL: Failed to configure uinput device: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!(
                "FATAL: Failed to open /dev/uinput: {}. \
                 Ensure uinput module is loaded and you have write permission.",
                e
            );
            std::process::exit(1);
        }
    };

    for device in &mut keyboard_devices {
        if let Err(e) = device.grab() {
            eprintln!(
                "FATAL: Failed to grab device '{}': {}",
                device.name().unwrap_or("?"),
                e
            );
            std::process::exit(1);
        }
        println!("[OK] Grabbed device: {}", device.name().unwrap_or("?"));
    }

    let pressed_mods: Arc<parking_lot::Mutex<HashSet<u16>>> =
        Arc::new(parking_lot::Mutex::new(HashSet::new()));

    println!("\nKeyboard is now LOCKED. Most keys will be blocked.");
    println!("Press Ctrl+Alt+L to toggle lock/unlock.");
    println!("Press Ctrl+C in this console to quit.\n");

    let pressed_mods_clone = pressed_mods.clone();

    let handle = thread::spawn(move || {
        let mut events_buf: Vec<InputEvent> = Vec::new();

        loop {
            for device in &mut keyboard_devices {
                let events: Vec<InputEvent> = match device.fetch_events() {
                    Ok(iter) => iter.collect(),
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("[WARN] Error reading events: {}", e);
                        }
                        continue;
                    }
                };

                for ev in events {
                    let ev_type = ev.event_type();

                    if ev_type != EventType::KEY {
                        events_buf.push(ev);
                        continue;
                    }

                    let code = ev.code();
                    let value = ev.value();
                    let is_down = value == 1;
                    let is_up = value == 0;

                    if is_down || is_up {
                        let mut mods = pressed_mods_clone.lock();
                        if is_ctrl(code) {
                            if is_down {
                                mods.insert(code);
                            } else {
                                mods.remove(&code);
                            }
                        }
                        if is_alt(code) {
                            if is_down {
                                mods.insert(code);
                            } else {
                                mods.remove(&code);
                            }
                        }
                    }

                    let locked = LOCKED.load(Ordering::SeqCst);

                    if is_down && locked {
                        let mods = pressed_mods_clone.lock();
                        let ctrl = mods.contains(&KEY_LEFTCTRL)
                            || mods.contains(&KEY_RIGHTCTRL);
                        let alt = mods.contains(&KEY_LEFTALT)
                            || mods.contains(&KEY_RIGHTALT);

                        if ctrl && alt && code == KEY_L {
                            LOCKED.store(false, Ordering::SeqCst);
                            println!("\n[UNLOCKED] Keyboard unlocked! Press Ctrl+Alt+L to re-lock.");
                            events_buf.push(ev);
                            continue;
                        }

                        if is_modifier(code) {
                            events_buf.push(ev);
                            continue;
                        }

                        println!(
                            "[BLOCKED] Key {}  code=0x{:02X}",
                            key_name(code),
                            code
                        );
                        continue;
                    }

                    if is_down && !locked {
                        let mods = pressed_mods_clone.lock();
                        let ctrl = mods.contains(&KEY_LEFTCTRL)
                            || mods.contains(&KEY_RIGHTCTRL);
                        let alt = mods.contains(&KEY_LEFTALT)
                            || mods.contains(&KEY_RIGHTALT);

                        if ctrl && alt && code == KEY_L {
                            LOCKED.store(true, Ordering::SeqCst);
                            println!("\n[LOCKED] Keyboard locked. Press Ctrl+Alt+L to unlock.");
                            events_buf.push(ev);
                            continue;
                        }
                    }

                    events_buf.push(ev);
                }
            }

            if !events_buf.is_empty() {
                let _ = virtual_dev.emit(&events_buf);
                events_buf.clear();
            }

            thread::sleep(Duration::from_millis(1));
        }
    });

    handle.join().unwrap();
}
