#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform;
#[cfg(target_os = "macos")]
pub use macos::MacOSPlatform;
#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionStatus {
    Granted,
    Denied { reason: String, fix_command: Option<String> },
}

pub trait PlatformExtras: Send + Sync {
    fn get_foreground_process(&self) -> Option<String>;
    fn check_permissions(&self) -> PermissionStatus;
    fn open_permission_settings(&self);
}

#[cfg(target_os = "windows")]
pub fn create_platform() -> WindowsPlatform {
    WindowsPlatform::new()
}

#[cfg(target_os = "macos")]
pub fn create_platform() -> MacOSPlatform {
    MacOSPlatform::new()
}

#[cfg(target_os = "linux")]
pub fn create_platform() -> LinuxPlatform {
    LinuxPlatform::new()
}