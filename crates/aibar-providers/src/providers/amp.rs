use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const AMP_HOST: &str = "ampcode.com";
const AMP_SESSION_COOKIE: &str = "amp_session";
const AMP_USAGE_API: &str = "https://ampcode.com/api/usage";

/// Strategy that reads Amp session from browser cookie.
pub struct AmpCookieStrategy;

#[async_trait]
impl FetchStrategy for AmpCookieStrategy {
    fn id(&self) -> &str {
        "amp.cookie"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Web
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        let profiles = crate::auth::browser_detect::detect_browser_profiles();
        !profiles.is_empty()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let cookie = find_amp_cookie().await?;

        let client = ctx.http_client.clone();
        let response = client
            .get(AMP_USAGE_API)
            .header("Cookie", format!("{}={}", AMP_SESSION_COOKIE, cookie))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Amp API returned status {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;

        // TODO: Parse actual Amp usage response
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
        false
    }
}

async fn find_amp_cookie() -> anyhow::Result<String> {
    crate::auth::cookie_finder::find_browser_cookie(AMP_HOST, AMP_SESSION_COOKIE).await
}
