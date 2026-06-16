use super::{PermissionStatus, PlatformExtras};

pub struct MacOSPlatform;

impl MacOSPlatform {
    pub fn new() -> Self {
        MacOSPlatform
    }
}

impl PlatformExtras for MacOSPlatform {
    fn get_foreground_process(&self) -> Option<String> {
        // macOS: requires Input Monitoring permission
        // Simplified stub - would use NSWorkspace via objc2
        None
    }

    fn check_permissions(&self) -> PermissionStatus {
        PermissionStatus::Denied {
            reason: "Input Monitoring permission required".into(),
            fix_command: Some(
                "Open System Preferences > Privacy & Security > Input Monitoring".into(),
            ),
            can_auto_fix: false,
        }
    }

    fn open_permission_settings(&self) {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }
}
