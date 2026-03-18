use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

pub struct OllamaLocalStrategy;

#[async_trait]
impl FetchStrategy for OllamaLocalStrategy {
    fn id(&self) -> &str {
        "ollama.local"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::LocalProbe
    }

    async fn is_available(&self, ctx: &FetchContext) -> bool {
        // Check if Ollama is running locally
        ctx.http_client.get("http://localhost:11434/api/version")
            .send()
            .await
            .is_ok()
    }

    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let response = ctx.http_client.get("http://localhost:11434/api/ps").send().await?;
        let body: serde_json::Value = response.json().await?;

        let models = body["models"]
            .as_array()
            .map(|m| m.len())
            .unwrap_or(0);

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
                    plan: Some(format!("{} model(s) running", models)),
                }),
            },
            credits: None,
            source_label: "local".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        false
    }
}
