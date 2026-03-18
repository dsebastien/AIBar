use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

/// Vertex AI usage fetch via gcloud CLI credentials and local cost logs.
pub struct VertexAiCliStrategy;

#[async_trait]
impl FetchStrategy for VertexAiCliStrategy {
    fn id(&self) -> &str {
        "vertex-ai.cli"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        // Check if gcloud CLI is installed
        tokio::process::Command::new("gcloud")
            .arg("--version")
            .output()
            .await
            .is_ok()
    }

    async fn fetch(&self, _ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        // TODO: Parse gcloud auth credentials and query Vertex AI usage API
        // 1. Run `gcloud auth print-access-token` to get OAuth token
        // 2. Query Vertex AI quota/usage API
        // 3. Optionally parse local Claude cost logs for Vertex AI entries

        let output = tokio::process::Command::new("gcloud")
            .args(["auth", "print-access-token"])
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("gcloud auth failed: not authenticated");
        }

        let _token = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // TODO: Use token to query Vertex AI usage
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
                    plan: Some("Vertex AI (gcloud)".to_string()),
                }),
            },
            credits: None,
            source_label: "gcloud-cli".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        true
    }
}
