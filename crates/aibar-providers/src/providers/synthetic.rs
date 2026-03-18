use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const SYNTHETIC_API_TOKEN_ENV: &str = "SYNTHETIC_API_KEY";
const SYNTHETIC_API_BASE: &str = "https://api.synthetic.com/v1";

pub struct SyntheticApiTokenStrategy;

#[async_trait]
impl FetchStrategy for SyntheticApiTokenStrategy {
    fn id(&self) -> &str {
        "synthetic.api_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiToken
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        ctx.has_env(SYNTHETIC_API_TOKEN_ENV)
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = ctx.require_env(SYNTHETIC_API_TOKEN_ENV)?;

        let client = ctx.http_client.clone();
        let response = client
            .get(format!("{}/account/usage", SYNTHETIC_API_BASE))
            .header("Authorization", format!("Bearer {}", token))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;

        // TODO: Parse actual Synthetic API usage response
        let used = body["usage"]["requests_used"].as_f64().unwrap_or(0.0);
        let limit = body["usage"]["requests_limit"].as_f64().unwrap_or(1.0);
        let used_percent = if limit > 0.0 {
            (used / limit) * 100.0
        } else {
            0.0
        };

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: None,
                    resets_at: None,
                    reset_description: body["usage"]["reset_description"]
                        .as_str()
                        .map(|s| s.to_string()),
                }),
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity: body["account"]["email"].as_str().map(|email| {
                    crate::models::ProviderIdentitySnapshot {
                        email: Some(email.to_string()),
                        team: None,
                        plan: body["account"]["plan"].as_str().map(|s| s.to_string()),
                    }
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
