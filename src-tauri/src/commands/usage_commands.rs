use crate::managers::refresh_manager::RefreshManager;
use crate::state::AppState;
use aibar_providers::models::{ProviderId, UsageSnapshot};
use std::collections::HashMap;
use tauri::{AppHandle, State};

/// Get all cached usage snapshots.
#[tauri::command]
pub async fn get_all_usage(
    state: State<'_, AppState>,
) -> Result<HashMap<String, UsageSnapshot>, String> {
    let snapshots = state.usage_snapshots.read().await;
    let result: HashMap<String, UsageSnapshot> = snapshots
        .iter()
        .map(|(id, snap)| {
            let key = serde_json::to_value(id)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| format!("{:?}", id).to_lowercase());
            (key, snap.clone())
        })
        .collect();
    Ok(result)
}

/// Refresh usage data for a single provider.
#[tauri::command]
pub async fn refresh_provider(
    provider_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let pid: ProviderId = provider_id.parse()?;
    RefreshManager::refresh_single(pid, &state.usage_snapshots, &app).await;
    Ok(())
}

/// Refresh usage data for all enabled providers.
#[tauri::command]
pub async fn refresh_all(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let providers = {
        let config = state.config.read().await;
        config.enabled_providers.clone()
    };
    RefreshManager::refresh_all(&providers, &state.usage_snapshots, &app).await;
    Ok(())
}
