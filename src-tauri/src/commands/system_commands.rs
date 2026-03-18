/// Open a URL in the system's default browser.
#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL '{}': {}", url, e))
}
