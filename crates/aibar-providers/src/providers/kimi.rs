use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const KIMI_HOST: &str = "kimi.moonshot.cn";
const KIMI_SESSION_COOKIE: &str = "session_id";
const KIMI_USAGE_API: &str = "https://kimi.moonshot.cn/api/user/usage";

// ---------------------------------------------------------------------------
// Cookie/JWT strategy: reads session cookie and parses JWT for rate limits
// ---------------------------------------------------------------------------

pub struct KimiCookieJwtStrategy;

#[async_trait]
impl FetchStrategy for KimiCookieJwtStrategy {
    fn id(&self) -> &str {
        "kimi.cookie_jwt"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Web
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        let profiles = crate::auth::browser_detect::detect_browser_profiles();
        !profiles.is_empty()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let cookie = find_kimi_cookie().await?;

        // Try to decode JWT from the session cookie to extract rate limit info
        let jwt_info = crate::auth::jwt::decode_jwt_payload(&cookie).ok();
        let exp = crate::auth::jwt::jwt_expiration(&cookie).ok().flatten();

        let resets_at = exp.and_then(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.with_timezone(&Utc))
        });

        // Fetch usage from the API
        let client = reqwest::Client::new();
        let response = client
            .get(KIMI_USAGE_API)
            .header("Cookie", format!("{}={}", KIMI_SESSION_COOKIE, cookie))
            .header("Referer", "https://kimi.moonshot.cn/")
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            // Try to parse rate limit info from response headers
            let headers = response.headers().clone();
            let status = response.status();

            if status.as_u16() == 429 {
                let rate_limit_remaining = headers
                    .get("x-ratelimit-remaining")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let rate_limit_total = headers
                    .get("x-ratelimit-limit")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(1.0);

                let used = rate_limit_total - rate_limit_remaining;
                let used_percent = if rate_limit_total > 0.0 {
                    (used / rate_limit_total) * 100.0
                } else {
                    100.0
                };

                let reset_time = headers
                    .get("x-ratelimit-reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<i64>().ok())
                    .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                    .map(|dt| dt.with_timezone(&Utc));

                return Ok(FetchResult {
                    usage: UsageSnapshot {
                        primary: Some(crate::models::RateWindow {
                            used_percent,
                            window_minutes: None,
                            resets_at: reset_time,
                            reset_description: Some("Rate limited".to_string()),
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
                });
            }

            anyhow::bail!("Kimi API returned status {}", status);
        }

        let body: serde_json::Value = response.json().await?;

        // Parse usage response
        let used = body["used"].as_f64().unwrap_or(0.0);
        let limit = body["limit"].as_f64().unwrap_or(1.0);
        let used_percent = if limit > 0.0 {
            (used / limit) * 100.0
        } else {
            0.0
        };

        let identity = jwt_info.map(|jwt| crate::models::ProviderIdentitySnapshot {
            email: jwt["email"].as_str().map(|s| s.to_string()),
            team: None,
            plan: jwt["plan"].as_str().map(|s| s.to_string()),
        });

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: body["window_minutes"].as_i64().map(|m| m as i32),
                    resets_at,
                    reset_description: body["reset_description"]
                        .as_str()
                        .map(|s| s.to_string()),
                }),
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity,
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

async fn find_kimi_cookie() -> anyhow::Result<String> {
    let profiles = crate::auth::browser_detect::detect_browser_profiles();

    for profile in &profiles {
        match profile.browser {
            crate::auth::browser_detect::Browser::Firefox => {
                if let Ok(Some(val)) = crate::auth::cookie_firefox::read_firefox_cookies(
                    &profile.profile_path,
                    KIMI_HOST,
                    KIMI_SESSION_COOKIE,
                ) {
                    return Ok(val);
                }
            }
            _ => {
                #[cfg(target_os = "linux")]
                if let Ok(Some(val)) = crate::auth::cookie_chrome_linux::read_chrome_cookie(
                    &profile.profile_path,
                    KIMI_HOST,
                    KIMI_SESSION_COOKIE,
                )
                .await
                {
                    return Ok(val);
                }
                #[cfg(target_os = "windows")]
                if let Ok(Some(val)) = crate::auth::cookie_chrome_windows::read_chrome_cookie(
                    &profile.profile_path,
                    KIMI_HOST,
                    KIMI_SESSION_COOKIE,
                ) {
                    return Ok(val);
                }
            }
        }
    }

    anyhow::bail!("Kimi session cookie not found in any browser")
}
