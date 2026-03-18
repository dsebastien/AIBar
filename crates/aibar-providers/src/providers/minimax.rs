use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const MINIMAX_API_TOKEN_ENV: &str = "MINIMAX_API_KEY";
const MINIMAX_GROUP_ID_ENV: &str = "MINIMAX_GROUP_ID";
const MINIMAX_API_BASE: &str = "https://api.minimax.chat/v1";

/// Strategy that fetches MiniMax usage via API token.
pub struct MiniMaxApiTokenStrategy;

#[async_trait]
impl FetchStrategy for MiniMaxApiTokenStrategy {
    fn id(&self) -> &str {
        "minimax.api_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiToken
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        ctx.env.contains_key(MINIMAX_API_TOKEN_ENV)
            || std::env::var(MINIMAX_API_TOKEN_ENV).is_ok()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = ctx
            .env
            .get(MINIMAX_API_TOKEN_ENV)
            .cloned()
            .or_else(|| std::env::var(MINIMAX_API_TOKEN_ENV).ok())
            .ok_or_else(|| anyhow::anyhow!("MINIMAX_API_KEY not set"))?;

        let group_id = ctx
            .env
            .get(MINIMAX_GROUP_ID_ENV)
            .cloned()
            .or_else(|| std::env::var(MINIMAX_GROUP_ID_ENV).ok())
            .unwrap_or_default();

        let client = reqwest::Client::new();
        let url = if group_id.is_empty() {
            format!("{}/billing/usage", MINIMAX_API_BASE)
        } else {
            format!(
                "{}/billing/usage?group_id={}",
                MINIMAX_API_BASE, group_id
            )
        };

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;

        // TODO: Parse actual MiniMax billing/usage response
        let used = body["data"]["total_used"].as_f64().unwrap_or(0.0);
        let limit = body["data"]["total_limit"].as_f64().unwrap_or(1.0);
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
                    reset_description: None,
                }),
                secondary: None,
                tertiary: None,
                provider_cost: Some(crate::models::ProviderCostSnapshot {
                    used,
                    limit,
                    currency_code: "CNY".to_string(),
                    period: Some("monthly".to_string()),
                    resets_at: None,
                }),
                updated_at: Utc::now(),
                identity: None,
            },
            credits: Some(crate::models::CreditsSnapshot {
                remaining: limit - used,
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
