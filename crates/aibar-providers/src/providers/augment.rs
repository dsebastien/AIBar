use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const AUGMENT_HOST: &str = "augmentcode.com";
const AUGMENT_SESSION_COOKIE: &str = "augment_session";
const AUGMENT_USAGE_API: &str = "https://api.augmentcode.com/v1/usage";

/// Strategy that reads Augment session from browser cookie.
pub struct AugmentCookieStrategy;

#[async_trait]
impl FetchStrategy for AugmentCookieStrategy {
    fn id(&self) -> &str {
        "augment.cookie"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Web
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        let profiles = crate::auth::browser_detect::detect_browser_profiles();
        !profiles.is_empty()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let cookie = find_augment_cookie().await?;

        let client = reqwest::Client::new();
        let response = client
            .get(AUGMENT_USAGE_API)
            .header("Cookie", format!("{}={}", AUGMENT_SESSION_COOKIE, cookie))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Augment API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;

        // TODO: Parse actual Augment usage response
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
                        team: body["user"]["team"].as_str().map(|s| s.to_string()),
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
        true // Fall back to CLI strategy
    }
}

/// Strategy that uses the Augment CLI to fetch usage info.
pub struct AugmentCliStrategy;

#[async_trait]
impl FetchStrategy for AugmentCliStrategy {
    fn id(&self) -> &str {
        "augment.cli"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        which_augment().is_some()
    }

    async fn fetch(&self, _ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let binary = which_augment()
            .ok_or_else(|| anyhow::anyhow!("augment CLI not found"))?;

        let output = tokio::process::Command::new(&binary)
            .arg("status")
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!(
                "augment CLI exited with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // TODO: Parse actual augment CLI status output
        let used_percent = parse_augment_cli_output(&stdout);

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: None,
                    resets_at: None,
                    reset_description: Some(stdout.trim().to_string()),
                }),
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity: None,
            },
            credits: None,
            source_label: "cli".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        false
    }
}

fn which_augment() -> Option<String> {
    let candidates = ["augment", "augment-code"];
    for name in &candidates {
        if let Ok(output) = std::process::Command::new("which")
            .arg(name)
            .output()
        {
            if output.status.success() {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn parse_augment_cli_output(output: &str) -> f64 {
    // TODO: Parse real augment CLI output format
    // Look for patterns like "Usage: 42/500" or "42% used"
    for line in output.lines() {
        if let Some(pct) = line
            .split_whitespace()
            .find(|w| w.ends_with('%'))
            .and_then(|w| w.trim_end_matches('%').parse::<f64>().ok())
        {
            return pct;
        }
    }
    0.0
}

async fn find_augment_cookie() -> anyhow::Result<String> {
    let profiles = crate::auth::browser_detect::detect_browser_profiles();

    for profile in &profiles {
        match profile.browser {
            crate::auth::browser_detect::Browser::Firefox => {
                if let Ok(Some(val)) = crate::auth::cookie_firefox::read_firefox_cookies(
                    &profile.profile_path,
                    AUGMENT_HOST,
                    AUGMENT_SESSION_COOKIE,
                ) {
                    return Ok(val);
                }
            }
            _ => {
                #[cfg(target_os = "linux")]
                if let Ok(Some(val)) = crate::auth::cookie_chrome_linux::read_chrome_cookie(
                    &profile.profile_path,
                    AUGMENT_HOST,
                    AUGMENT_SESSION_COOKIE,
                )
                .await
                {
                    return Ok(val);
                }
                #[cfg(target_os = "windows")]
                if let Ok(Some(val)) = crate::auth::cookie_chrome_windows::read_chrome_cookie(
                    &profile.profile_path,
                    AUGMENT_HOST,
                    AUGMENT_SESSION_COOKIE,
                ) {
                    return Ok(val);
                }
            }
        }
    }

    anyhow::bail!("Augment session cookie not found in any browser")
}
