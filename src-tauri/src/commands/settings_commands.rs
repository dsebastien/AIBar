use crate::managers::settings_manager::SettingsManager;
use crate::state::AppState;
use aibar_providers::models::{AppConfig, ProviderId};
use tauri::{AppHandle, State};

/// Get the current application settings.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.read().await;
    Ok(config.clone())
}

/// Update the application settings.
#[tauri::command]
pub async fn update_settings(
    config: AppConfig,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    // Save to persistent store
    SettingsManager::save_config(&app, &config)?;

    // Update in-memory state
    {
        let mut current = state.config.write().await;
        *current = config;
    }

    Ok(())
}

/// Enable or disable a specific provider.
#[tauri::command]
pub async fn toggle_provider(
    provider_id: String,
    enabled: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let pid: ProviderId = provider_id.parse()?;

    let mut config = state.config.write().await;

    if enabled {
        if !config.enabled_providers.contains(&pid) {
            config.enabled_providers.push(pid);
        }
    } else {
        config.enabled_providers.retain(|&p| p != pid);
    }

    // Save updated config
    SettingsManager::save_config(&app, &config)?;

    Ok(())
}
