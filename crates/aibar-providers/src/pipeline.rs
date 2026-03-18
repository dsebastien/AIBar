use crate::models::ProviderId;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchAttempt {
    pub strategy_id: String,
    pub kind: FetchKind,
    pub was_available: bool,
    pub error: Option<String>,
}

pub struct FetchOutcome {
    pub result: anyhow::Result<FetchResult>,
    pub attempts: Vec<FetchAttempt>,
}

pub async fn run_pipeline(
    provider: ProviderId,
    strategies: &[Box<dyn FetchStrategy>],
    ctx: &FetchContext,
) -> FetchOutcome {
    let mut attempts = Vec::with_capacity(strategies.len());

    for strategy in strategies {
        let available = strategy.is_available(ctx).await;
        if !available {
            attempts.push(FetchAttempt {
                strategy_id: strategy.id().to_string(),
                kind: strategy.kind(),
                was_available: false,
                error: None,
            });
            continue;
        }

        match strategy.fetch(ctx).await {
            Ok(result) => {
                attempts.push(FetchAttempt {
                    strategy_id: strategy.id().to_string(),
                    kind: strategy.kind(),
                    was_available: true,
                    error: None,
                });
                return FetchOutcome {
                    result: Ok(result),
                    attempts,
                };
            }
            Err(e) => {
                let desc = e.to_string();
                let should_fallback = strategy.should_fallback(&e, ctx);
                attempts.push(FetchAttempt {
                    strategy_id: strategy.id().to_string(),
                    kind: strategy.kind(),
                    was_available: true,
                    error: Some(desc),
                });
                if !should_fallback {
                    return FetchOutcome {
                        result: Err(e),
                        attempts,
                    };
                }
            }
        }
    }

    FetchOutcome {
        result: Err(anyhow::anyhow!("No available strategy for {}", provider)),
        attempts,
    }
}
