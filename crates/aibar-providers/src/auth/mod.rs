pub mod browser_detect;
pub mod cookie_firefox;
pub mod jwt;

#[cfg(target_os = "windows")]
pub mod cookie_chrome_windows;

#[cfg(target_os = "linux")]
pub mod cookie_chrome_linux;
