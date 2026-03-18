use aibar_providers::models::{ProviderId, RefreshCadence, UsageSnapshot};
use aibar_providers::pipeline;
use aibar_providers::providers::ollama::OllamaLocalStrategy;
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
    pub fn start(&self, enabled_providers: Arc<RwLock<Vec<ProviderId>>>) {
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
                                let p = enabled_providers.read().await;
                                p.clone()
                            };
                            Self::do_refresh_all(&providers, &snapshots, &app_handle).await;
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

    /// Refresh usage data for all given providers.
    pub async fn refresh_all(
        providers: &[ProviderId],
        snapshots: &Arc<RwLock<HashMap<ProviderId, UsageSnapshot>>>,
        app_handle: &AppHandle,
    ) {
        Self::do_refresh_all(providers, snapshots, app_handle).await;
    }

    async fn do_refresh_all(
        providers: &[ProviderId],
        snapshots: &Arc<RwLock<HashMap<ProviderId, UsageSnapshot>>>,
        app_handle: &AppHandle,
    ) {
        info!("RefreshManager: refreshing {} providers", providers.len());
        for &provider_id in providers {
            Self::refresh_single(provider_id, snapshots, app_handle).await;
        }
    }
}

/// Build the list of fetch strategies for a given provider.
///
/// As providers are implemented, this function will grow. Currently only Ollama
/// has a concrete strategy.
fn build_strategies(provider_id: ProviderId) -> Vec<Box<dyn aibar_providers::traits::FetchStrategy>> {
    match provider_id {
        ProviderId::Ollama => {
            vec![Box::new(OllamaLocalStrategy)]
        }
        // Other providers will have strategies added as they are implemented.
        _ => vec![],
    }
}
