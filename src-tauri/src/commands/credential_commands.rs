use tauri_plugin_store::StoreExt;
use tauri::AppHandle;

const CREDENTIALS_STORE: &str = "credentials.json";

/// Store an API token for a provider.
#[tauri::command]
pub async fn store_api_token(
    provider_id: String,
    token: String,
    app: AppHandle,
) -> Result<(), String> {
    let store = app
        .store(CREDENTIALS_STORE)
        .map_err(|e| format!("Failed to open credential store: {}", e))?;

    let key = format!("token_{}", provider_id);
    store.set(&key, serde_json::Value::String(token));

    store
        .save()
        .map_err(|e| format!("Failed to save credential store: {}", e))?;

    Ok(())
}

/// Delete a stored credential for a provider.
#[tauri::command]
pub async fn delete_credential(provider_id: String, app: AppHandle) -> Result<(), String> {
    let store = app
        .store(CREDENTIALS_STORE)
        .map_err(|e| format!("Failed to open credential store: {}", e))?;

    let key = format!("token_{}", provider_id);
    let _ = store.delete(&key);

    store
        .save()
        .map_err(|e| format!("Failed to save credential store: {}", e))?;

    Ok(())
}

/// Check whether a credential exists for a provider.
/// Returns "stored" if a token exists, "missing" otherwise.
#[tauri::command]
pub async fn get_credential_status(
    provider_id: String,
    app: AppHandle,
) -> Result<String, String> {
    let store = app
        .store(CREDENTIALS_STORE)
        .map_err(|e| format!("Failed to open credential store: {}", e))?;

    let key = format!("token_{}", provider_id);
    let status = match store.get(&key) {
        Some(_) => "stored".to_string(),
        None => "missing".to_string(),
    };

    Ok(status)
}
