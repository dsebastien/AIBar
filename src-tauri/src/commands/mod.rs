pub mod credential_commands;
pub mod settings_commands;
pub mod system_commands;
pub mod usage_commands;

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

// Re-export all commands for convenient access outside the module.
// These are used by lib.rs via full paths in generate_handler!, but re-exported
// here for ergonomic use elsewhere.
#[allow(unused_imports)]
pub use credential_commands::{delete_credential, get_credential_status, store_api_token};
#[allow(unused_imports)]
pub use settings_commands::{get_settings, toggle_provider, update_settings};
#[allow(unused_imports)]
pub use system_commands::open_url;
#[allow(unused_imports)]
pub use usage_commands::{get_all_usage, refresh_all, refresh_provider};
