use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

const KIMI_K2_API_TOKEN_ENV: &str = "KIMI_K2_API_KEY";
const KIMI_K2_API_BASE: &str = "https://api.moonshot.cn/v1";

pub struct KimiK2ApiTokenStrategy;

#[async_trait]
impl FetchStrategy for KimiK2ApiTokenStrategy {
    fn id(&self) -> &str {
        "kimi_k2.api_token"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiToken
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        ctx.has_env(KIMI_K2_API_TOKEN_ENV)
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let token = ctx.require_env(KIMI_K2_API_TOKEN_ENV)?;

        let client = ctx.http_client.clone();
        let response = client
            .get(format!("{}/users/me/balance", KIMI_K2_API_BASE))
            .header("Authorization", format!("Bearer {}", token))
            .timeout(ctx.web_timeout)
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;

        // Moonshot API returns balance information
        let available_balance = body["data"]["available_balance"]
            .as_f64()
            .unwrap_or(0.0);
        let total_balance = body["data"]["total_balance"]
            .as_f64()
            .unwrap_or(1.0);
        let used_percent = if total_balance > 0.0 {
            ((total_balance - available_balance) / total_balance) * 100.0
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
                    used: total_balance - available_balance,
                    limit: total_balance,
                    currency_code: "CNY".to_string(),
                    period: None,
                    resets_at: None,
                }),
                updated_at: Utc::now(),
                identity: None,
            },
            credits: Some(crate::models::CreditsSnapshot {
                remaining: available_balance,
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
