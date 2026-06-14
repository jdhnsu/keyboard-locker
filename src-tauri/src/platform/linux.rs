use super::{PermissionStatus, PlatformExtras};

pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        LinuxPlatform
    }
}

impl PlatformExtras for LinuxPlatform {
    fn get_foreground_process(&self) -> Option<String> {
        // Linux X11: requires accessing root window properties
        // Wayland: not reliably possible
        None
    }

    fn check_permissions(&self) -> PermissionStatus {
        // Check if user is in the input group
        let in_group = std::process::Command::new("groups")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("input"))
            .unwrap_or(false);

        if in_group {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Denied {
                reason: "User is not in the input group".into(),
                fix_command: Some("sudo usermod -a -G input $USER && newgrp input".into()),
            }
        }
    }

    fn open_permission_settings(&self) {
        // No GUI permission panel; show terminal instructions
    }
}