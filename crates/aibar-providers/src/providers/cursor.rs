use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;

const CURSOR_SETTINGS_API: &str = "https://www.cursor.com/api/usage";
const CURSOR_SESSION_COOKIE_NAME: &str = "WorkosCursorSessionToken";
const CURSOR_HOST: &str = "cursor.com";

/// Reads the Cursor session token from the local Cursor config.
fn read_cursor_session_token() -> Option<String> {
    let config_dir = cursor_config_dir()?;
    let storage_path = config_dir.join("User").join("globalStorage").join("storage.json");
    if !storage_path.exists() {
        return None;
    }
    let contents = std::fs::read_to_string(&storage_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    json["cursorAuth/accessToken"]
        .as_str()
        .map(|s| s.to_string())
}

fn cursor_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config").join("Cursor"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .ok()
            .map(|h| PathBuf::from(h).join("Cursor"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

/// Strategy that reads Cursor session token from local storage.
pub struct CursorLocalTokenStrategy;

#[async_trait]
impl FetchStrategy for CursorLocalTokenStrategy {
    fn id(&self) -> &str {
        "cursor.local_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Web
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        read_cursor_session_token().is_some()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = read_cursor_session_token()
            .ok_or_else(|| anyhow::anyhow!("Cursor session token not found"))?;

        let client = reqwest::Client::new();
        let response = client
            .get(CURSOR_SETTINGS_API)
            .header("Cookie", format!("{}={}", CURSOR_SESSION_COOKIE_NAME, token))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Cursor API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;

        // Cursor usage API response:
        // { "gpt-4": { "numRequests": 42, "maxRequestUsage": 500, ... }, ... }
        let premium_used = body["gpt-4"]["numRequests"]
            .as_f64()
            .or_else(|| body["gpt-4o"]["numRequests"].as_f64())
            .unwrap_or(0.0);
        let premium_limit = body["gpt-4"]["maxRequestUsage"]
            .as_f64()
            .or_else(|| body["gpt-4o"]["maxRequestUsage"].as_f64())
            .unwrap_or(500.0);

        let used_percent = if premium_limit > 0.0 {
            (premium_used / premium_limit) * 100.0
        } else {
            0.0
        };

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: None,
                    resets_at: None,
                    reset_description: Some(format!(
                        "{:.0}/{:.0} premium requests",
                        premium_used, premium_limit
                    )),
                }),
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity: None,
            },
            credits: None,
            source_label: "local".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        true // Fall back to cookie strategy
    }
}

/// Strategy that reads Cursor session from browser cookie.
pub struct CursorCookieStrategy;

#[async_trait]
impl FetchStrategy for CursorCookieStrategy {
    fn id(&self) -> &str {
        "cursor.cookie"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Web
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        // Check if any browser profiles exist
        let profiles = crate::auth::browser_detect::detect_browser_profiles();
        !profiles.is_empty()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let profiles = crate::auth::browser_detect::detect_browser_profiles();
        let mut cookie_value: Option<String> = None;

        for profile in &profiles {
            match profile.browser {
                crate::auth::browser_detect::Browser::Firefox => {
                    if let Ok(Some(val)) = crate::auth::cookie_firefox::read_firefox_cookies(
                        &profile.profile_path,
                        CURSOR_HOST,
                        CURSOR_SESSION_COOKIE_NAME,
                    ) {
                        cookie_value = Some(val);
                        break;
                    }
                }
                _ => {
                    #[cfg(target_os = "linux")]
                    if let Ok(Some(val)) =
                        crate::auth::cookie_chrome_linux::read_chrome_cookie(
                            &profile.profile_path,
                            CURSOR_HOST,
                            CURSOR_SESSION_COOKIE_NAME,
                        )
                        .await
                    {
                        cookie_value = Some(val);
                        break;
                    }
                    #[cfg(target_os = "windows")]
                    if let Ok(Some(val)) =
                        crate::auth::cookie_chrome_windows::read_chrome_cookie(
                            &profile.profile_path,
                            CURSOR_HOST,
                            CURSOR_SESSION_COOKIE_NAME,
                        )
                    {
                        cookie_value = Some(val);
                        break;
                    }
                }
            }
        }

        let cookie = cookie_value
            .ok_or_else(|| anyhow::anyhow!("Cursor session cookie not found in any browser"))?;

        let client = reqwest::Client::new();
        let response = client
            .get(CURSOR_SETTINGS_API)
            .header(
                "Cookie",
                format!("{}={}", CURSOR_SESSION_COOKIE_NAME, cookie),
            )
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Cursor API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;

        let premium_used = body["gpt-4"]["numRequests"]
            .as_f64()
            .or_else(|| body["gpt-4o"]["numRequests"].as_f64())
            .unwrap_or(0.0);
        let premium_limit = body["gpt-4"]["maxRequestUsage"]
            .as_f64()
            .or_else(|| body["gpt-4o"]["maxRequestUsage"].as_f64())
            .unwrap_or(500.0);

        let used_percent = if premium_limit > 0.0 {
            (premium_used / premium_limit) * 100.0
        } else {
            0.0
        };

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: None,
                    resets_at: None,
                    reset_description: Some(format!(
                        "{:.0}/{:.0} premium requests",
                        premium_used, premium_limit
                    )),
                }),
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity: None,
            },
            credits: None,
            source_label: "cookie".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        false
    }
}
