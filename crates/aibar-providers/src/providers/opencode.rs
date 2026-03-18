use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const OPENCODE_HOST: &str = "opencode.ai";
const OPENCODE_SESSION_COOKIE: &str = "opencode_session";
const OPENCODE_USAGE_API: &str = "https://api.opencode.ai/v1/usage";

/// Strategy that reads OpenCode session from browser cookie.
pub struct OpenCodeCookieStrategy;

#[async_trait]
impl FetchStrategy for OpenCodeCookieStrategy {
    fn id(&self) -> &str {
        "opencode.cookie"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Web
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        let profiles = crate::auth::browser_detect::detect_browser_profiles();
        !profiles.is_empty()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let cookie = find_opencode_cookie().await?;

        let client = ctx.http_client.clone();
        let response = client
            .get(OPENCODE_USAGE_API)
            .header(
                "Cookie",
                format!("{}={}", OPENCODE_SESSION_COOKIE, cookie),
            )
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("OpenCode API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;

        // TODO: Parse actual OpenCode usage response
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

async fn find_opencode_cookie() -> anyhow::Result<String> {
    crate::auth::cookie_finder::find_browser_cookie(OPENCODE_HOST, OPENCODE_SESSION_COOKIE).await
}
