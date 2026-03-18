use aibar_providers::models::{AppConfig, ProviderId, UsageSnapshot};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state managed by Tauri.
pub struct AppState {
    /// Cached usage snapshots for each provider.
    pub usage_snapshots: Arc<RwLock<HashMap<ProviderId, UsageSnapshot>>>,
    /// Current application configuration.
    pub config: Arc<RwLock<AppConfig>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            usage_snapshots: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(AppConfig::default())
    }
}
