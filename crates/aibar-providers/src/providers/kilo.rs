use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const KILO_HOST: &str = "kilo.dev";
const KILO_SESSION_COOKIE: &str = "kilo_session";
const KILO_USAGE_API: &str = "https://api.kilo.dev/v1/usage";

/// Strategy that reads Kilo session from browser cookie.
pub struct KiloCookieStrategy;

#[async_trait]
impl FetchStrategy for KiloCookieStrategy {
    fn id(&self) -> &str {
        "kilo.cookie"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Web
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        let profiles = crate::auth::browser_detect::detect_browser_profiles();
        !profiles.is_empty()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let cookie = find_kilo_cookie().await?;

        let client = reqwest::Client::new();
        let response = client
            .get(KILO_USAGE_API)
            .header("Cookie", format!("{}={}", KILO_SESSION_COOKIE, cookie))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Kilo API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;

        // TODO: Parse actual Kilo usage response
        let used_percent = body["usage"]["used_percent"]
            .as_f64()
            .unwrap_or(0.0);

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: None,
                    resets_at: None,
                    reset_description: body["usage"]["description"]
                        .as_str()
                        .map(|s| s.to_string()),
                }),
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity: body["user"]["email"].as_str().map(|email| {
                    crate::models::ProviderIdentitySnapshot {
                        email: Some(email.to_string()),
                        team: None,
                        plan: body["user"]["plan"].as_str().map(|s| s.to_string()),
                    }
                }),
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

async fn find_kilo_cookie() -> anyhow::Result<String> {
    let profiles = crate::auth::browser_detect::detect_browser_profiles();

    for profile in &profiles {
        match profile.browser {
            crate::auth::browser_detect::Browser::Firefox => {
                if let Ok(Some(val)) = crate::auth::cookie_firefox::read_firefox_cookies(
                    &profile.profile_path,
                    KILO_HOST,
                    KILO_SESSION_COOKIE,
                ) {
                    return Ok(val);
                }
            }
            _ => {
                #[cfg(target_os = "linux")]
                if let Ok(Some(val)) = crate::auth::cookie_chrome_linux::read_chrome_cookie(
                    &profile.profile_path,
                    KILO_HOST,
                    KILO_SESSION_COOKIE,
                )
                .await
                {
                    return Ok(val);
                }
                #[cfg(target_os = "windows")]
                if let Ok(Some(val)) = crate::auth::cookie_chrome_windows::read_chrome_cookie(
                    &profile.profile_path,
                    KILO_HOST,
                    KILO_SESSION_COOKIE,
                ) {
                    return Ok(val);
                }
            }
        }
    }

    anyhow::bail!("Kilo session cookie not found in any browser")
}
