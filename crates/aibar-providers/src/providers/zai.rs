use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const ZAI_API_TOKEN_ENV: &str = "ZAI_API_KEY";
const ZAI_API_BASE: &str = "https://api.z.ai/v1";

pub struct ZaiApiTokenStrategy;

#[async_trait]
impl FetchStrategy for ZaiApiTokenStrategy {
    fn id(&self) -> &str {
        "zai.api_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiToken
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        ctx.env.contains_key(ZAI_API_TOKEN_ENV)
            || std::env::var(ZAI_API_TOKEN_ENV).is_ok()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = ctx
            .env
            .get(ZAI_API_TOKEN_ENV)
            .cloned()
            .or_else(|| std::env::var(ZAI_API_TOKEN_ENV).ok())
            .ok_or_else(|| anyhow::anyhow!("ZAI_API_KEY not set"))?;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/usage", ZAI_API_BASE))
            .header("Authorization", format!("Bearer {}", token))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;

        // TODO: Parse actual z.ai usage response structure
        let used_percent = body["usage"]["percent_used"]
            .as_f64()
            .unwrap_or(0.0);

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: None,
                    resets_at: None,
                    reset_description: body["usage"]["resets_at"]
                        .as_str()
                        .map(|s| s.to_string()),
                }),
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity: body["email"].as_str().map(|email| {
                    crate::models::ProviderIdentitySnapshot {
                        email: Some(email.to_string()),
                        team: body["team"].as_str().map(|s| s.to_string()),
                        plan: body["plan"].as_str().map(|s| s.to_string()),
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
