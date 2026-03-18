use crate::auth::browser_detect::{detect_browser_profiles, Browser};
use crate::auth::cookie_firefox;

pub async fn find_browser_cookie(host: &str, cookie_name: &str) -> anyhow::Result<String> {
    let profiles = detect_browser_profiles();

    for profile in &profiles {
        match profile.browser {
            Browser::Firefox => {
                if let Ok(Some(value)) = cookie_firefox::read_firefox_cookies(&profile.profile_path, host, cookie_name) {
                    return Ok(value);
                }
            }
            _ => {
                #[cfg(target_os = "linux")]
                {
                    if let Ok(Some(value)) = crate::auth::cookie_chrome_linux::read_chrome_cookie(&profile.profile_path, host, cookie_name).await {
                        return Ok(value);
                    }
                }
                #[cfg(target_os = "windows")]
                {
                    if let Ok(Some(value)) = crate::auth::cookie_chrome_windows::read_chrome_cookie(&profile.profile_path, host, cookie_name).await {
                        return Ok(value);
                    }
                }
            }
        }
    }

    anyhow::bail!("{} cookie '{}' not found in any browser", host, cookie_name)
}
