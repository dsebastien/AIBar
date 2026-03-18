use aibar_providers::models::ProviderId;
use aibar_providers::registry;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: ProviderId,
    pub display_name: String,
    pub session_label: String,
    pub weekly_label: String,
    pub supports_credits: bool,
    pub default_enabled: bool,
    pub dashboard_url: Option<String>,
    pub color: String,
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn get_all_providers() -> Vec<ProviderInfo> {
    ProviderId::all()
        .iter()
        .map(|&id| {
            let desc = registry::get_descriptor(id);
            ProviderInfo {
                id,
                display_name: desc.metadata.display_name.to_string(),
                session_label: desc.metadata.session_label.to_string(),
                weekly_label: desc.metadata.weekly_label.to_string(),
                supports_credits: desc.metadata.supports_credits,
                default_enabled: desc.metadata.default_enabled,
                dashboard_url: desc.metadata.dashboard_url.map(|s| s.to_string()),
                color: format!(
                    "#{:02x}{:02x}{:02x}",
                    desc.branding.color.r, desc.branding.color.g, desc.branding.color.b
                ),
            }
        })
        .collect()
}
