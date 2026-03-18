use crate::models::{CreditsSnapshot, UsageSnapshot};
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const OPENROUTER_API_TOKEN_ENV: &str = "OPENROUTER_API_KEY";
const OPENROUTER_AUTH_KEY_URL: &str = "https://openrouter.ai/api/v1/auth/key";

pub struct OpenRouterApiTokenStrategy;

#[async_trait]
impl FetchStrategy for OpenRouterApiTokenStrategy {
    fn id(&self) -> &str {
        "openrouter.api_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiToken
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        ctx.has_env(OPENROUTER_API_TOKEN_ENV)
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = ctx.require_env(OPENROUTER_API_TOKEN_ENV)?;

        let client = ctx.http_client.clone();
        let response = client
            .get(OPENROUTER_AUTH_KEY_URL)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;

        // OpenRouter /api/v1/auth/key response:
        // { "data": { "label": "...", "usage": 0.5, "limit": 10.0, "is_free_tier": false, ... } }
        let data = &body["data"];
        let usage_usd = data["usage"].as_f64().unwrap_or(0.0);
        let limit_usd = data["limit"].as_f64();
        let is_free_tier = data["is_free_tier"].as_bool().unwrap_or(false);

        let used_percent = match limit_usd {
            Some(limit) if limit > 0.0 => (usage_usd / limit) * 100.0,
            _ => 0.0,
        };

        let remaining = limit_usd.map(|l| l - usage_usd).unwrap_or(0.0);

        let plan_name = if is_free_tier {
            "Free Tier".to_string()
        } else {
            "Paid".to_string()
        };

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: None,
                    resets_at: None,
                    reset_description: None,
                }),
                secondary: None,
                tertiary: None,
                provider_cost: limit_usd.map(|limit| crate::models::ProviderCostSnapshot {
                    used: usage_usd,
                    limit,
                    currency_code: "USD".to_string(),
                    period: None,
                    resets_at: None,
                }),
                updated_at: Utc::now(),
                identity: Some(crate::models::ProviderIdentitySnapshot {
                    email: None,
                    team: None,
                    plan: Some(plan_name),
                }),
            },
            credits: Some(CreditsSnapshot {
                remaining,
                updated_at: Utc::now(),
            }),
            source_label: "api".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        false
    }
}
