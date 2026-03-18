use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const OPENAI_API_TOKEN_ENV: &str = "OPENAI_API_KEY";
const OPENAI_USAGE_API: &str = "https://api.openai.com/v1/dashboard/billing/usage";
const OPENAI_SUBSCRIPTION_API: &str = "https://api.openai.com/v1/dashboard/billing/subscription";

// ---------------------------------------------------------------------------
// OAuth strategy: reads Codex/OpenAI OAuth token from local config
// ---------------------------------------------------------------------------

pub struct CodexOAuthStrategy;

fn codex_config_dir() -> Option<std::path::PathBuf> {
    crate::cli_helpers::app_config_dir("codex", "codex")
}

fn read_codex_oauth_token() -> Option<String> {
    let config_dir = codex_config_dir()?;
    let auth_path = config_dir.join("auth.json");
    if !auth_path.exists() {
        return None;
    }
    let contents = std::fs::read_to_string(&auth_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    json["accessToken"]
        .as_str()
        .or_else(|| json["token"].as_str())
        .map(|s| s.to_string())
}

#[async_trait]
impl FetchStrategy for CodexOAuthStrategy {
    fn id(&self) -> &str {
        "codex.oauth"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::OAuth
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        read_codex_oauth_token().is_some()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = read_codex_oauth_token()
            .ok_or_else(|| anyhow::anyhow!("Codex OAuth token not found"))?;

        fetch_openai_usage(&token, ctx, self.id(), self.kind(), "oauth").await
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// RPC/API token strategy
// ---------------------------------------------------------------------------

pub struct CodexApiTokenStrategy;

#[async_trait]
impl FetchStrategy for CodexApiTokenStrategy {
    fn id(&self) -> &str {
        "codex.api_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiToken
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        ctx.has_env(OPENAI_API_TOKEN_ENV)
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = ctx.require_env(OPENAI_API_TOKEN_ENV)?;

        fetch_openai_usage(&token, ctx, self.id(), self.kind(), "api").await
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// CLI strategy: parses `codex` CLI output
// ---------------------------------------------------------------------------

pub struct CodexCliStrategy;

#[async_trait]
impl FetchStrategy for CodexCliStrategy {
    fn id(&self) -> &str {
        "codex.cli"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        which_codex().is_some()
    }

    async fn fetch(&self, _ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let binary = which_codex()
            .ok_or_else(|| anyhow::anyhow!("codex CLI not found"))?;

        let output = tokio::process::Command::new(&binary)
            .arg("usage")
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!(
                "codex CLI exited with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Try parsing as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            let used = json["total_usage"].as_f64().unwrap_or(0.0) / 100.0; // cents to dollars
            let limit = json["hard_limit_usd"].as_f64().unwrap_or(120.0);
            let used_percent = if limit > 0.0 {
                (used / limit) * 100.0
            } else {
                0.0
            };

            return Ok(FetchResult {
                usage: UsageSnapshot {
                    primary: Some(crate::models::RateWindow {
                        used_percent,
                        window_minutes: None,
                        resets_at: None,
                        reset_description: Some(format!("${:.2}/${:.2}", used, limit)),
                    }),
                    secondary: None,
                    tertiary: None,
                    provider_cost: Some(crate::models::ProviderCostSnapshot {
                        used,
                        limit,
                        currency_code: "USD".to_string(),
                        period: Some("monthly".to_string()),
                        resets_at: None,
                    }),
                    updated_at: Utc::now(),
                    identity: None,
                },
                credits: None,
                source_label: "cli".to_string(),
                strategy_id: self.id().to_string(),
                strategy_kind: self.kind(),
            });
        }

        // Text output fallback
        let used_percent = crate::cli_helpers::parse_percent_from_text(&stdout);

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

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn which_codex() -> Option<String> {
    crate::cli_helpers::which_cli(&["codex", "openai"])
}

async fn fetch_openai_usage(
    token: &str,
    ctx: &FetchContext,
    strategy_id: &str,
    strategy_kind: FetchKind,
    source_label: &str,
) -> anyhow::Result<FetchResult> {
    let client = ctx.http_client.clone();

    // Fetch subscription info (limits)
    let sub_response = client
        .get(OPENAI_SUBSCRIPTION_API)
        .header("Authorization", format!("Bearer {}", token))
        .timeout(ctx.web_timeout)
        .send()
        .await?;

    let sub_body: serde_json::Value = sub_response.json().await?;
    let hard_limit = sub_body["hard_limit_usd"].as_f64().unwrap_or(120.0);
    let plan_name = sub_body["plan"]["title"]
        .as_str()
        .map(|s| s.to_string());

    // Fetch current month usage
    let now = Utc::now();
    let start_date = now.format("%Y-%m-01").to_string();
    let end_date = now.format("%Y-%m-%d").to_string();

    let usage_response = client
        .get(OPENAI_USAGE_API)
        .query(&[("start_date", &start_date), ("end_date", &end_date)])
        .header("Authorization", format!("Bearer {}", token))
        .timeout(ctx.web_timeout)
        .send()
        .await?;

    let usage_body: serde_json::Value = usage_response.json().await?;
    let total_usage_cents = usage_body["total_usage"].as_f64().unwrap_or(0.0);
    let used_usd = total_usage_cents / 100.0;

    let used_percent = if hard_limit > 0.0 {
        (used_usd / hard_limit) * 100.0
    } else {
        0.0
    };

    Ok(FetchResult {
        usage: UsageSnapshot {
            primary: Some(crate::models::RateWindow {
                used_percent,
                window_minutes: None,
                resets_at: None,
                reset_description: Some(format!("${:.2}/${:.2}", used_usd, hard_limit)),
            }),
            secondary: None,
            tertiary: None,
            provider_cost: Some(crate::models::ProviderCostSnapshot {
                used: used_usd,
                limit: hard_limit,
                currency_code: "USD".to_string(),
                period: Some("monthly".to_string()),
                resets_at: None,
            }),
            updated_at: Utc::now(),
            identity: Some(crate::models::ProviderIdentitySnapshot {
                email: sub_body["account_name"]
                    .as_str()
                    .map(|s| s.to_string()),
                team: None,
                plan: plan_name,
            }),
        },
        credits: Some(crate::models::CreditsSnapshot {
            remaining: hard_limit - used_usd,
            updated_at: Utc::now(),
        }),
        source_label: source_label.to_string(),
        strategy_id: strategy_id.to_string(),
        strategy_kind,
    })
}
