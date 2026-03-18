use aibar_providers::models::AppConfig;
use log::{error, info};
use serde_json;
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tokio::sync::RwLock;

const STORE_FILE: &str = "settings.json";
const CONFIG_KEY: &str = "app_config";

/// Manages persistent application settings using tauri-plugin-store.
pub struct SettingsManager;

impl SettingsManager {
    /// Load the application config from the store, or return defaults.
    pub fn load_config(app: &AppHandle) -> AppConfig {
        let store = match app.store(STORE_FILE) {
            Ok(s) => s,
            Err(e) => {
                error!("SettingsManager: failed to open store: {}", e);
                return AppConfig::default();
            }
        };

        match store.get(CONFIG_KEY) {
            Some(value) => {
                match serde_json::from_value::<AppConfig>(value.clone()) {
                    Ok(config) => {
                        info!("SettingsManager: loaded config from store");
                        config
                    }
                    Err(e) => {
                        error!("SettingsManager: failed to deserialize config: {}", e);
                        AppConfig::default()
                    }
                }
            }
            None => {
                info!("SettingsManager: no saved config, using defaults");
                AppConfig::default()
            }
        }
    }

    /// Save the application config to the store.
    pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
        let store = app
            .store(STORE_FILE)
            .map_err(|e| format!("Failed to open store: {}", e))?;

        let value = serde_json::to_value(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        store.set(CONFIG_KEY, value);

        store
            .save()
            .map_err(|e| format!("Failed to save store: {}", e))?;

        info!("SettingsManager: config saved to store");
        Ok(())
    }

    /// Get the current config from the shared state.
    pub async fn get_config(config: &Arc<RwLock<AppConfig>>) -> AppConfig {
        config.read().await.clone()
    }
}
