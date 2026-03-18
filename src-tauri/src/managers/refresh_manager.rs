use aibar_providers::models::{AppConfig, ProviderId, RefreshCadence, UsageSnapshot};
use aibar_providers::pipeline;
use aibar_providers::providers::{
    amp::AmpCookieStrategy,
    antigravity::AntigravityCookieStrategy,
    augment::{AugmentCliStrategy, AugmentCookieStrategy},
    claude::{ClaudeCliStrategy, ClaudeOAuthStrategy, ClaudeWebCookieStrategy},
    codex::{CodexApiTokenStrategy, CodexCliStrategy, CodexOAuthStrategy},
    copilot::{CopilotApiTokenStrategy, CopilotDeviceFlowStrategy},
    cursor::{CursorCookieStrategy, CursorLocalTokenStrategy},
    factory::FactoryCookieStrategy,
    gemini::{GeminiApiTokenStrategy, GeminiCliStrategy},
    jetbrains::JetBrainsLocalConfigStrategy,
    kilo::KiloCookieStrategy,
    kimi::KimiCookieJwtStrategy,
    kimi_k2::KimiK2ApiTokenStrategy,
    kiro::KiroCliStrategy,
    minimax::MiniMaxApiTokenStrategy,
    ollama::OllamaLocalStrategy,
    opencode::OpenCodeCookieStrategy,
    openrouter::OpenRouterApiTokenStrategy,
    synthetic::SyntheticApiTokenStrategy,
    vertex_ai::VertexAiCliStrategy,
    warp::WarpCookieStrategy,
    zai::ZaiApiTokenStrategy,
};
use aibar_providers::traits::FetchContext;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tokio::time::{self, Duration};
use tokio_util::sync::CancellationToken;

/// Event name emitted to the frontend when usage data is updated.
pub const USAGE_UPDATED_EVENT: &str = "usage-updated";

/// Manages periodic background refresh of provider usage data.
pub struct RefreshManager {
    usage_snapshots: Arc<RwLock<HashMap<ProviderId, UsageSnapshot>>>,
    config_cadence: Arc<RwLock<RefreshCadence>>,
    cancel_token: CancellationToken,
    app_handle: AppHandle,
}

impl RefreshManager {
    /// Create a new RefreshManager. Does not start the background task automatically.
    pub fn new(
        usage_snapshots: Arc<RwLock<HashMap<ProviderId, UsageSnapshot>>>,
        cadence: RefreshCadence,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            usage_snapshots,
            config_cadence: Arc::new(RwLock::new(cadence)),
            cancel_token: CancellationToken::new(),
            app_handle,
        }
    }

    /// Start the background refresh loop.
    pub fn start(&self, config: Arc<RwLock<AppConfig>>) {
        let snapshots = self.usage_snapshots.clone();
        let cadence = self.config_cadence.clone();
        let cancel = self.cancel_token.clone();
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            info!("RefreshManager: background task started");
            loop {
                let interval_secs = {
                    let c = cadence.read().await;
                    c.to_seconds()
                };

                let sleep_duration = match interval_secs {
                    Some(secs) => Duration::from_secs(secs),
                    None => {
                        // Manual mode: sleep for a long time and wait for cancellation
                        // or cadence change.
                        Duration::from_secs(3600)
                    }
                };

                tokio::select! {
                    _ = cancel.cancelled() => {
                        info!("RefreshManager: background task cancelled");
                        break;
                    }
                    _ = time::sleep(sleep_duration) => {
                        // Only refresh if not in manual mode
                        if interval_secs.is_some() {
                            let providers = {
                                let cfg = config.read().await;
                                cfg.enabled_providers.clone()
                            };
                            Self::refresh_all(&providers, &snapshots, &app_handle).await;
                        }
                    }
                }
            }
        });
    }

    /// Stop the background refresh loop.
    pub fn stop(&self) {
        self.cancel_token.cancel();
    }

    /// Update the refresh cadence. Takes effect on the next tick.
    pub async fn set_cadence(&self, cadence: RefreshCadence) {
        let mut c = self.config_cadence.write().await;
        *c = cadence;
        info!("RefreshManager: cadence updated to {:?}", cadence);
    }

    /// Refresh usage data for a single provider.
    pub async fn refresh_single(
        provider_id: ProviderId,
        snapshots: &Arc<RwLock<HashMap<ProviderId, UsageSnapshot>>>,
        app_handle: &AppHandle,
    ) {
        let strategies = build_strategies(provider_id);
        if strategies.is_empty() {
            warn!(
                "RefreshManager: no strategies available for {:?}",
                provider_id
            );
            return;
        }

        let ctx = FetchContext::default();
        let outcome = pipeline::run_pipeline(provider_id, &strategies, &ctx).await;

        match outcome.result {
            Ok(fetch_result) => {
                let snapshot = fetch_result.usage;
                {
                    let mut map = snapshots.write().await;
                    map.insert(provider_id, snapshot.clone());
                }

                let mut payload = HashMap::new();
                payload.insert(provider_id, snapshot);

                if let Err(e) = app_handle.emit(USAGE_UPDATED_EVENT, &payload) {
                    error!("RefreshManager: failed to emit event: {}", e);
                }
            }
            Err(e) => {
                warn!(
                    "RefreshManager: failed to fetch {:?}: {}",
                    provider_id, e
                );
                for attempt in &outcome.attempts {
                    if let Some(ref err) = attempt.error {
                        warn!(
                            "  strategy={} kind={:?} available={} error={}",
                            attempt.strategy_id, attempt.kind, attempt.was_available, err
                        );
                    }
                }
            }
        }
    }

    /// Refresh usage data for all given providers concurrently.
    pub async fn refresh_all(
        providers: &[ProviderId],
        snapshots: &Arc<RwLock<HashMap<ProviderId, UsageSnapshot>>>,
        app_handle: &AppHandle,
    ) {
        info!(
            "RefreshManager: refreshing {} providers concurrently",
            providers.len()
        );
        let futures: Vec<_> = providers
            .iter()
            .map(|&pid| Self::refresh_single(pid, snapshots, app_handle))
            .collect();
        futures::future::join_all(futures).await;
    }
}

/// Build the list of fetch strategies for a given provider.
fn build_strategies(provider_id: ProviderId) -> Vec<Box<dyn aibar_providers::traits::FetchStrategy>> {
    match provider_id {
        ProviderId::Ollama => vec![Box::new(OllamaLocalStrategy)],
        ProviderId::Claude => vec![
            Box::new(ClaudeOAuthStrategy),
            Box::new(ClaudeCliStrategy),
            Box::new(ClaudeWebCookieStrategy),
        ],
        ProviderId::Codex => vec![
            Box::new(CodexOAuthStrategy),
            Box::new(CodexApiTokenStrategy),
            Box::new(CodexCliStrategy),
        ],
        ProviderId::Cursor => vec![
            Box::new(CursorLocalTokenStrategy),
            Box::new(CursorCookieStrategy),
        ],
        ProviderId::Gemini => vec![
            Box::new(GeminiCliStrategy),
            Box::new(GeminiApiTokenStrategy),
        ],
        ProviderId::Copilot => vec![
            Box::new(CopilotDeviceFlowStrategy),
            Box::new(CopilotApiTokenStrategy),
        ],
        ProviderId::Augment => vec![
            Box::new(AugmentCookieStrategy),
            Box::new(AugmentCliStrategy),
        ],
        ProviderId::Amp => vec![Box::new(AmpCookieStrategy)],
        ProviderId::Kimi => vec![Box::new(KimiCookieJwtStrategy)],
        ProviderId::KimiK2 => vec![Box::new(KimiK2ApiTokenStrategy)],
        ProviderId::Zai => vec![Box::new(ZaiApiTokenStrategy)],
        ProviderId::MiniMax => vec![Box::new(MiniMaxApiTokenStrategy)],
        ProviderId::Factory => vec![Box::new(FactoryCookieStrategy)],
        ProviderId::JetBrains => vec![Box::new(JetBrainsLocalConfigStrategy)],
        ProviderId::Kilo => vec![Box::new(KiloCookieStrategy)],
        ProviderId::Kiro => vec![Box::new(KiroCliStrategy)],
        ProviderId::VertexAi => vec![Box::new(VertexAiCliStrategy)],
        ProviderId::Synthetic => vec![Box::new(SyntheticApiTokenStrategy)],
        ProviderId::Warp => vec![Box::new(WarpCookieStrategy)],
        ProviderId::OpenRouter => vec![Box::new(OpenRouterApiTokenStrategy)],
        ProviderId::Antigravity => vec![Box::new(AntigravityCookieStrategy)],
        ProviderId::OpenCode => vec![Box::new(OpenCodeCookieStrategy)],
    }
}
