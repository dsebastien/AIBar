use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;

const COPILOT_API_TOKEN_ENV: &str = "GITHUB_COPILOT_TOKEN";
const GITHUB_TOKEN_ENV: &str = "GITHUB_TOKEN";
const COPILOT_USAGE_API: &str = "https://api.github.com/copilot/usage";
const COPILOT_BILLING_API: &str = "https://api.github.com/user/copilot_billing/usage";

// ---------------------------------------------------------------------------
// Device flow / GitHub token strategy
// ---------------------------------------------------------------------------

pub struct CopilotDeviceFlowStrategy;

fn copilot_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config").join("github-copilot"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(|h| PathBuf::from(h).join("github-copilot"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

fn read_copilot_token() -> Option<String> {
    // Try reading from GitHub Copilot's local config (VS Code extension stores tokens here)
    let config_dir = copilot_config_dir()?;
    let hosts_path = config_dir.join("hosts.json");
    if hosts_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&hosts_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                // hosts.json format: { "github.com": { "oauth_token": "gho_..." } }
                if let Some(token) = json["github.com"]["oauth_token"].as_str() {
                    return Some(token.to_string());
                }
            }
        }
    }

    // Also try the apps.json path used by newer versions
    let apps_path = config_dir.join("apps.json");
    if apps_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&apps_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(token) = json["github.com"]["oauth_token"].as_str() {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}

#[async_trait]
impl FetchStrategy for CopilotDeviceFlowStrategy {
    fn id(&self) -> &str {
        "copilot.device_flow"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::OAuth
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        read_copilot_token().is_some()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = read_copilot_token()
            .ok_or_else(|| anyhow::anyhow!("Copilot device flow token not found"))?;

        fetch_copilot_usage(&token, ctx, self.id(), self.kind(), "device_flow").await
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// API token strategy (GITHUB_TOKEN or GITHUB_COPILOT_TOKEN env var)
// ---------------------------------------------------------------------------

pub struct CopilotApiTokenStrategy;

#[async_trait]
impl FetchStrategy for CopilotApiTokenStrategy {
    fn id(&self) -> &str {
        "copilot.api_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiToken
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        ctx.env.contains_key(COPILOT_API_TOKEN_ENV)
            || ctx.env.contains_key(GITHUB_TOKEN_ENV)
            || std::env::var(COPILOT_API_TOKEN_ENV).is_ok()
            || std::env::var(GITHUB_TOKEN_ENV).is_ok()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = ctx
            .env
            .get(COPILOT_API_TOKEN_ENV)
            .cloned()
            .or_else(|| ctx.env.get(GITHUB_TOKEN_ENV).cloned())
            .or_else(|| std::env::var(COPILOT_API_TOKEN_ENV).ok())
            .or_else(|| std::env::var(GITHUB_TOKEN_ENV).ok())
            .ok_or_else(|| {
                anyhow::anyhow!("Neither GITHUB_COPILOT_TOKEN nor GITHUB_TOKEN is set")
            })?;

        fetch_copilot_usage(&token, ctx, self.id(), self.kind(), "api").await
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

async fn fetch_copilot_usage(
    token: &str,
    ctx: &FetchContext,
    strategy_id: &str,
    strategy_kind: FetchKind,
    source_label: &str,
) -> anyhow::Result<FetchResult> {
    let client = reqwest::Client::new();

    // Try the copilot billing/usage endpoint first
    let response = client
        .get(COPILOT_BILLING_API)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "AIBar")
        .timeout(ctx.web_timeout)
        .send()
        .await;

    let body = match response {
        Ok(resp) if resp.status().is_success() => resp.json::<serde_json::Value>().await?,
        _ => {
            // Fallback to the general usage endpoint
            let resp2 = client
                .get(COPILOT_USAGE_API)
                .header("Authorization", format!("Bearer {}", token))
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .header("User-Agent", "AIBar")
                .timeout(ctx.web_timeout)
                .send()
                .await?;

            if !resp2.status().is_success() {
                anyhow::bail!("GitHub Copilot API returned status {}", resp2.status());
            }

            resp2.json::<serde_json::Value>().await?
        }
    };

    // Parse usage data
    let premium_used = body["premium_requests_used"]
        .as_f64()
        .unwrap_or(0.0);
    let premium_limit = body["premium_requests_limit"]
        .as_f64()
        .unwrap_or(300.0);
    let chat_used = body["chat_requests_used"]
        .as_f64()
        .unwrap_or(0.0);
    let chat_limit = body["chat_requests_limit"]
        .as_f64()
        .unwrap_or(0.0);

    let premium_percent = if premium_limit > 0.0 {
        (premium_used / premium_limit) * 100.0
    } else {
        0.0
    };

    let chat_percent = if chat_limit > 0.0 {
        (chat_used / chat_limit) * 100.0
    } else {
        0.0
    };

    Ok(FetchResult {
        usage: UsageSnapshot {
            primary: Some(crate::models::RateWindow {
                used_percent: premium_percent,
                window_minutes: None,
                resets_at: None,
                reset_description: Some(format!(
                    "{:.0}/{:.0} premium requests",
                    premium_used, premium_limit
                )),
            }),
            secondary: if chat_limit > 0.0 {
                Some(crate::models::RateWindow {
                    used_percent: chat_percent,
                    window_minutes: None,
                    resets_at: None,
                    reset_description: Some(format!(
                        "{:.0}/{:.0} chat requests",
                        chat_used, chat_limit
                    )),
                })
            } else {
                None
            },
            tertiary: None,
            provider_cost: None,
            updated_at: Utc::now(),
            identity: Some(crate::models::ProviderIdentitySnapshot {
                email: None,
                team: body["organization"].as_str().map(|s| s.to_string()),
                plan: body["copilot_plan"].as_str().map(|s| s.to_string()),
            }),
        },
        credits: None,
        source_label: source_label.to_string(),
        strategy_id: strategy_id.to_string(),
        strategy_kind,
    })
}
