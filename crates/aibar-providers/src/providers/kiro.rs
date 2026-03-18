use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;

/// Strategy that uses the Kiro CLI to fetch usage information.
pub struct KiroCliStrategy;

#[async_trait]
impl FetchStrategy for KiroCliStrategy {
    fn id(&self) -> &str {
        "kiro.cli"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        which_kiro().is_some()
    }

    async fn fetch(&self, _ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let binary = which_kiro()
            .ok_or_else(|| anyhow::anyhow!("kiro CLI not found"))?;

        let output = tokio::process::Command::new(&binary)
            .arg("status")
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!(
                "kiro CLI exited with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let used_percent = parse_kiro_cli_output(&stdout);

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary: Some(crate::models::RateWindow {
                    used_percent,
                    window_minutes: None,
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
        false
    }
}

fn which_kiro() -> Option<String> {
    crate::cli_helpers::which_cli(&["kiro"])
}

fn parse_kiro_cli_output(output: &str) -> f64 {
    // Try percentage first
    let pct = crate::cli_helpers::parse_percent_from_text(output);
    if pct > 0.0 {
        return pct;
    }
    // Try fraction pattern "X/Y"
    for line in output.lines() {
        for word in line.split_whitespace() {
            let parts: Vec<&str> = word.split('/').collect();
            if parts.len() == 2 {
                if let (Ok(used), Ok(total)) =
                    (parts[0].parse::<f64>(), parts[1].parse::<f64>())
                {
                    if total > 0.0 {
                        return (used / total) * 100.0;
                    }
                }
            }
        }
    }
    0.0
}
