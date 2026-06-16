use super::{PermissionStatus, PlatformExtras};

pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Self {
        WindowsPlatform
    }
}

impl PlatformExtras for WindowsPlatform {
    fn get_foreground_process(&self) -> Option<String> {
        get_foreground_process_windows()
    }

    fn check_permissions(&self) -> PermissionStatus {
        PermissionStatus::Granted
    }

    fn open_permission_settings(&self) {}
}

fn get_foreground_process_windows() -> Option<String> {
    unsafe {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId,
        };

        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }

        let mut buf = [0u16; 260];
        let mut len = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            0, // PROCESS_NAME_WIN32 = 0
            buf.as_mut_ptr(),
            &mut len,
        );
        CloseHandle(handle);

        if result == 0 || len == 0 {
            return None;
        }

        let path = String::from_utf16_lossy(&buf[..len as usize]);
        std::path::Path::new(&path)
            .file_name()
            .and_then(|f| f.to_str())
            .map(|s| s.to_string())
    }
}
