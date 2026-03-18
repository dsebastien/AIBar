use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const GEMINI_API_TOKEN_ENV: &str = "GEMINI_API_KEY";

/// Strategy that uses the Gemini CLI to fetch usage information.
pub struct GeminiCliStrategy;

#[async_trait]
impl FetchStrategy for GeminiCliStrategy {
    fn id(&self) -> &str {
        "gemini.cli"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        which_gemini().is_some()
    }

    async fn fetch(&self, _ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let binary = which_gemini()
            .ok_or_else(|| anyhow::anyhow!("gemini CLI not found"))?;

        let output = tokio::process::Command::new(&binary)
            .arg("usage")
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!(
                "gemini CLI exited with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let used_percent = parse_gemini_cli_output(&stdout);

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: Some(1440), // Daily window
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
        true // Fall back to API strategy
    }
}

/// Strategy that uses the Gemini API key to fetch usage via REST.
pub struct GeminiApiTokenStrategy;

#[async_trait]
impl FetchStrategy for GeminiApiTokenStrategy {
    fn id(&self) -> &str {
        "gemini.api_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiToken
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        ctx.has_env(GEMINI_API_TOKEN_ENV)
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let api_key = ctx.require_env(GEMINI_API_TOKEN_ENV)?;

        // Use the Generative Language API to list models (validates the key)
        let client = ctx.http_client.clone();
        let response = client
            .get(format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            ))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Gemini API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        let model_count = body["models"]
            .as_array()
            .map(|m| m.len())
            .unwrap_or(0);

        // Gemini free tier doesn't have a usage API; we can only confirm the key works
        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: None,
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity: Some(crate::models::ProviderIdentitySnapshot {
                    email: None,
                    team: None,
                    plan: Some(format!("API key valid ({} models)", model_count)),
                }),
            },
            credits: None,
            source_label: "api".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        false
    }
}

fn which_gemini() -> Option<String> {
    crate::cli_helpers::which_cli(&["gemini"])
}

fn parse_gemini_cli_output(output: &str) -> f64 {
    // Try percentage first
    let pct = crate::cli_helpers::parse_percent_from_text(output);
    if pct > 0.0 {
        return pct;
    }
    // Try to find "X/Y" fraction pattern
    for line in output.lines() {
        if let Some((used, total)) = parse_fraction(line) {
            if total > 0.0 {
                return (used / total) * 100.0;
            }
        }
    }
    0.0
}

fn parse_fraction(line: &str) -> Option<(f64, f64)> {
    for word in line.split_whitespace() {
        let parts: Vec<&str> = word.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(a), Ok(b)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                return Some((a, b));
            }
        }
    }
    None
}
