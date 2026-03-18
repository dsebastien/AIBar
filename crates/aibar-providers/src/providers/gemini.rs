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
        ctx.env.contains_key(GEMINI_API_TOKEN_ENV)
            || std::env::var(GEMINI_API_TOKEN_ENV).is_ok()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let api_key = ctx
            .env
            .get(GEMINI_API_TOKEN_ENV)
            .cloned()
            .or_else(|| std::env::var(GEMINI_API_TOKEN_ENV).ok())
            .ok_or_else(|| anyhow::anyhow!("GEMINI_API_KEY not set"))?;

        // Use the Generative Language API to list models (validates the key)
        let client = reqwest::Client::new();
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
    if let Ok(output) = std::process::Command::new("which")
        .arg("gemini")
        .output()
    {
        if output.status.success() {
            return Some("gemini".to_string());
        }
    }
    None
}

fn parse_gemini_cli_output(output: &str) -> f64 {
    // TODO: Parse actual gemini CLI output format
    // Look for patterns like "42/1500 requests" or "42% used"
    for line in output.lines() {
        // Try to find percentage
        if let Some(pct) = line
            .split_whitespace()
            .find(|w| w.ends_with('%'))
            .and_then(|w| w.trim_end_matches('%').parse::<f64>().ok())
        {
            return pct;
        }
        // Try to find "X/Y" fraction pattern
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
