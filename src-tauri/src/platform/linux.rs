use super::{PermissionStatus, PlatformExtras};

pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        LinuxPlatform
    }
}

impl PlatformExtras for LinuxPlatform {
    fn get_foreground_process(&self) -> Option<String> {
        get_foreground_process_x11()
    }

    fn check_permissions(&self) -> PermissionStatus {
        let can_read_input = std::fs::read_dir("/dev/input")
            .ok()
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|entry| {
                    let name = entry.file_name();
                    if !name.to_string_lossy().starts_with("event") {
                        return false;
                    }
                    std::fs::File::open(entry.path()).is_ok()
                })
            })
            .unwrap_or(false);

        let uinput_writable = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/uinput")
            .is_ok();

        if can_read_input && uinput_writable {
            PermissionStatus::Granted
        } else {
            let mut reasons = Vec::new();
            if !can_read_input {
                reasons.push("无法访问键盘设备（权限不足）");
            }
            if !uinput_writable {
                reasons.push("无法写入 /dev/uinput（权限不足）");
            }
            PermissionStatus::Denied {
                reason: reasons.join("；"),
                fix_command: Some("pkexec bash -c 'modprobe uinput; chmod 0666 /dev/uinput /dev/input/event*; echo uinput > /etc/modules-load.d/uinput.conf; echo \"SUBSYSTEM==\\\"input\\\", MODE=\\\"0666\\\"\" > /etc/udev/rules.d/99-kbl-input.rules; echo \"KERNEL==\\\"uinput\\\", MODE=\\\"0666\\\"\" > /etc/udev/rules.d/99-kbl-uinput.rules; udevadm trigger; udevadm control --reload-rules'".into()),
                can_auto_fix: true,
            }
        }
    }

    fn try_fix_permissions(&self) -> bool {
        let script = r#"
modprobe uinput 2>/dev/null || true
chmod 0666 /dev/uinput 2>/dev/null || true
for dev in /dev/input/event*; do
    [ -e "$dev" ] && chmod 0666 "$dev" 2>/dev/null || true
done
echo 'uinput' > /etc/modules-load.d/uinput.conf 2>/dev/null || true
echo 'SUBSYSTEM=="input", MODE="0666"' > /etc/udev/rules.d/99-kbl-input.rules 2>/dev/null || true
echo 'KERNEL=="uinput", MODE="0666"' > /etc/udev/rules.d/99-kbl-uinput.rules 2>/dev/null || true
udevadm trigger 2>/dev/null || true
udevadm control --reload-rules 2>/dev/null || true
"#;
        let mut cmd = std::process::Command::new("pkexec");
        cmd.arg("bash").arg("-c").arg(script);
        if let Ok(display) = std::env::var("DISPLAY") {
            cmd.env("DISPLAY", display);
        }
        if let Ok(xauth) = std::env::var("XAUTHORITY") {
            cmd.env("XAUTHORITY", xauth);
        }
        let result = cmd.status();

        match result {
            Ok(status) if status.success() => {
                log::info!("pkexec fix permissions succeeded");
                std::thread::sleep(std::time::Duration::from_millis(500));
                true
            }
            Ok(status) => {
                log::warn!("pkexec fix permissions exited with: {}", status);
                false
            }
            Err(e) => {
                log::error!("pkexec failed to run: {}", e);
                false
            }
        }
    }

    fn open_permission_settings(&self) {}
}

fn get_foreground_process_x11() -> Option<String> {
    use std::ffi::CStr;

    let display_name = std::ffi::CString::new(":0").ok()?;

    unsafe {
        let display = x11::xlib::XOpenDisplay(display_name.as_ptr());
        if display.is_null() {
            return None;
        }

        let _root = x11::xlib::XDefaultRootWindow(display);

        let mut window: x11::xlib::Window = 0;
        let mut revert_to = 0;

        x11::xlib::XGetInputFocus(display, &mut window, &mut revert_to);

        if window == 0 || window == 1 {
            x11::xlib::XCloseDisplay(display);
            return None;
        }

        let mut pid: i64 = -1;
        let mut atom_type: x11::xlib::Atom = 0;
        let mut atom_format: i32 = 0;
        let mut n_items: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut prop: *mut u8 = std::ptr::null_mut();

        let net_pid = x11::xlib::XInternAtom(
            display,
            b"_NET_WM_PID\0".as_ptr() as *const i8,
            0,
        );

        let status = x11::xlib::XGetWindowProperty(
            display,
            window,
            net_pid,
            0,
            1,
            0,
            x11::xlib::XA_CARDINAL,
            &mut atom_type,
            &mut atom_format,
            &mut n_items,
            &mut bytes_after,
            &mut prop,
        );

        if status == 0 && n_items > 0 && !prop.is_null() {
            pid = *(prop as *const i64);
            x11::xlib::XFree(prop as *mut std::ffi::c_void);
        }

        let _name_ptr: *mut i8 = std::ptr::null_mut();
        let mut wm_name = x11::xlib::XTextProperty {
            value: std::ptr::null_mut(),
            encoding: 0,
            format: 0,
            nitems: 0,
        };

        let result = x11::xlib::XGetWMName(display, window, &mut wm_name);
        let _window_name = if result != 0 && !wm_name.value.is_null() {
            let s = CStr::from_ptr(wm_name.value as *const i8)
                .to_string_lossy()
                .into_owned();
            x11::xlib::XFree(wm_name.value as *mut std::ffi::c_void);
            Some(s)
        } else {
            None
        };

        let mut result_name = None;

        if pid > 0 {
            let proc_path = format!("/proc/{}/comm", pid);
            if let Ok(comm) = std::fs::read_to_string(&proc_path) {
                let comm = comm.trim().to_string();
                if !comm.is_empty() {
                    result_name = Some(comm);
                }
            }

            if result_name.is_none() {
                let exe_path = format!("/proc/{}/exe", pid);
                if let Ok(link) = std::fs::read_link(&exe_path) {
                    if let Some(name) = link.file_name().and_then(|n| n.to_str()) {
                        result_name = Some(name.to_string());
                    }
                }
            }
        }

        if result_name.is_none() {
            if let Some(name) = _window_name {
                result_name = Some(name);
            }
        }

        x11::xlib::XCloseDisplay(display);
        result_name
    }
}