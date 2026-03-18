use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BrowserProfile {
    pub browser: Browser,
    pub profile_path: PathBuf,
    pub profile_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    Chrome,
    Chromium,
    Firefox,
    Edge,
    Brave,
}

impl Browser {
    pub fn display_name(&self) -> &'static str {
        match self {
            Browser::Chrome => "Google Chrome",
            Browser::Chromium => "Chromium",
            Browser::Firefox => "Firefox",
            Browser::Edge => "Microsoft Edge",
            Browser::Brave => "Brave",
        }
    }
}

/// Detect installed browser profiles on the current platform.
pub fn detect_browser_profiles() -> Vec<BrowserProfile> {
    let mut profiles = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let home = dirs_next().unwrap_or_default();
        let chrome_paths = [
            (Browser::Chrome, home.join(".config/google-chrome")),
            (Browser::Chromium, home.join(".config/chromium")),
            (Browser::Edge, home.join(".config/microsoft-edge")),
            (Browser::Brave, home.join(".config/BraveSoftware/Brave-Browser")),
            (Browser::Firefox, home.join(".mozilla/firefox")),
        ];
        for (browser, base) in &chrome_paths {
            if base.exists() {
                profiles.push(BrowserProfile {
                    browser: *browser,
                    profile_path: base.join("Default"),
                    profile_name: "Default".to_string(),
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let local = PathBuf::from(local_app_data);
            let chrome_paths = [
                (Browser::Chrome, local.join("Google/Chrome/User Data")),
                (Browser::Edge, local.join("Microsoft/Edge/User Data")),
                (Browser::Brave, local.join("BraveSoftware/Brave-Browser/User Data")),
            ];
            for (browser, base) in &chrome_paths {
                if base.exists() {
                    profiles.push(BrowserProfile {
                        browser: *browser,
                        profile_path: base.join("Default"),
                        profile_name: "Default".to_string(),
                    });
                }
            }
        }
        if let Some(app_data) = std::env::var_os("APPDATA") {
            let roaming = PathBuf::from(app_data);
            let firefox_base = roaming.join("Mozilla/Firefox");
            if firefox_base.exists() {
                profiles.push(BrowserProfile {
                    browser: Browser::Firefox,
                    profile_path: firefox_base,
                    profile_name: "Default".to_string(),
                });
            }
        }
    }

    profiles
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
