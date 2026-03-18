use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const CLAUDE_HOST: &str = "claude.ai";
const CLAUDE_SESSION_COOKIE: &str = "sessionKey";
const CLAUDE_USAGE_API: &str = "https://claude.ai/api/organizations";

// ---------------------------------------------------------------------------
// OAuth strategy: reads Claude Code's OAuth token from local config
// ---------------------------------------------------------------------------

pub struct ClaudeOAuthStrategy;

fn claude_config_dir() -> Option<std::path::PathBuf> {
    crate::cli_helpers::app_config_dir("claude", "claude")
}

fn read_claude_oauth_token() -> Option<String> {
    let config_dir = claude_config_dir()?;
    // Claude Code stores OAuth credentials in ~/.config/claude/credentials.json
    let creds_path = config_dir.join("credentials.json");
    if !creds_path.exists() {
        return None;
    }
    let contents = std::fs::read_to_string(&creds_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    // Try different possible field names
    json["oauthToken"]
        .as_str()
        .or_else(|| json["accessToken"].as_str())
        .or_else(|| json["token"].as_str())
        .map(|s| s.to_string())
}

#[async_trait]
impl FetchStrategy for ClaudeOAuthStrategy {
    fn id(&self) -> &str {
        "claude.oauth"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::OAuth
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        read_claude_oauth_token().is_some()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = read_claude_oauth_token()
            .ok_or_else(|| anyhow::anyhow!("Claude OAuth token not found"))?;

        let client = ctx.http_client.clone();
        let response = client
            .get(CLAUDE_USAGE_API)
            .header("Authorization", format!("Bearer {}", token))
            .header("anthropic-client-platform", "aibar")
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Claude API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;

        let (usage, _identity) = parse_claude_org_response(&body);

        Ok(FetchResult {
            usage,
            credits: None,
            source_label: "oauth".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        true // Fall back to CLI or cookie
    }
}

// ---------------------------------------------------------------------------
// CLI strategy: parses `claude` CLI output
// ---------------------------------------------------------------------------

pub struct ClaudeCliStrategy;

#[async_trait]
impl FetchStrategy for ClaudeCliStrategy {
    fn id(&self) -> &str {
        "claude.cli"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        which_claude().is_some()
    }

    async fn fetch(&self, _ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let binary = which_claude()
            .ok_or_else(|| anyhow::anyhow!("claude CLI not found"))?;

        let output = tokio::process::Command::new(&binary)
            .args(["api", "usage"])
            .output()
            .await?;

        if !output.status.success() {
            // Try alternative subcommand
            let output2 = tokio::process::Command::new(&binary)
                .arg("--usage")
                .output()
                .await?;

            if !output2.status.success() {
                anyhow::bail!(
                    "claude CLI exited with status {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let stdout = String::from_utf8_lossy(&output2.stdout);
            return parse_claude_cli_result(&stdout, self);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_claude_cli_result(&stdout, self)
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        true // Fall back to web cookie
    }
}

// ---------------------------------------------------------------------------
// Web cookie strategy: reads session cookie from browser
// ---------------------------------------------------------------------------

pub struct ClaudeWebCookieStrategy;

#[async_trait]
impl FetchStrategy for ClaudeWebCookieStrategy {
    fn id(&self) -> &str {
        "claude.web_cookie"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Web
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        let profiles = crate::auth::browser_detect::detect_browser_profiles();
        !profiles.is_empty()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let cookie = find_claude_cookie().await?;

        let client = ctx.http_client.clone();
        let response = client
            .get(CLAUDE_USAGE_API)
            .header("Cookie", format!("{}={}", CLAUDE_SESSION_COOKIE, cookie))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Claude API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        let (usage, _identity) = parse_claude_org_response(&body);

        Ok(FetchResult {
            usage,
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

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn which_claude() -> Option<String> {
    crate::cli_helpers::which_cli(&["claude", "claude-code"])
}

fn parse_claude_org_response(
    body: &serde_json::Value,
) -> (UsageSnapshot, Option<crate::models::ProviderIdentitySnapshot>) {
    // Claude API /api/organizations returns an array of orgs.
    // Each org has usage information with rate limit data.
    let org = body
        .as_array()
        .and_then(|arr| arr.first())
        .unwrap_or(body);

    let rate_limit = &org["rate_limit"];
    let used = rate_limit["requests_used"].as_f64().unwrap_or(0.0);
    let limit = rate_limit["requests_limit"].as_f64().unwrap_or(1.0);
    let used_percent = if limit > 0.0 {
        (used / limit) * 100.0
    } else {
        0.0
    };

    let resets_at = rate_limit["resets_at"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let identity = Some(crate::models::ProviderIdentitySnapshot {
        email: org["email"].as_str().map(|s| s.to_string()),
        team: org["name"].as_str().map(|s| s.to_string()),
        plan: org["plan_type"].as_str().map(|s| s.to_string()),
    });

    let usage = UsageSnapshot {
        primary: Some(crate::models::RateWindow {
            used_percent,
            window_minutes: rate_limit["window_minutes"].as_i64().map(|m| m as i32),
            resets_at,
            reset_description: rate_limit["reset_description"]
                .as_str()
                .map(|s| s.to_string()),
        }),
        secondary: None,
        tertiary: None,
        provider_cost: None,
        updated_at: Utc::now(),
        identity: identity.clone(),
    };

    (usage, identity)
}

fn parse_claude_cli_result(
    stdout: &str,
    strategy: &dyn FetchStrategy,
) -> anyhow::Result<FetchResult> {
    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(stdout) {
        let (usage, _) = parse_claude_org_response(&json);
        return Ok(FetchResult {
            usage,
            credits: None,
            source_label: "cli".to_string(),
            strategy_id: strategy.id().to_string(),
            strategy_kind: strategy.kind(),
        });
    }

    // Fallback: parse text output
    let used_percent = crate::cli_helpers::parse_percent_from_text(stdout);

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
        strategy_id: strategy.id().to_string(),
        strategy_kind: strategy.kind(),
    })
}

async fn find_claude_cookie() -> anyhow::Result<String> {
    crate::auth::cookie_finder::find_browser_cookie(CLAUDE_HOST, CLAUDE_SESSION_COOKIE).await
}
