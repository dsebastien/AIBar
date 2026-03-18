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
        .map(|(id, snap)| (format!("{:?}", id).to_lowercase(), snap.clone()))
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
    let pid = parse_provider_id(&provider_id)?;
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

/// Parse a provider ID string into a ProviderId enum variant.
fn parse_provider_id(id: &str) -> Result<ProviderId, String> {
    serde_json::from_value(serde_json::Value::String(id.to_string()))
        .map_err(|_| format!("Unknown provider: {}", id))
}
