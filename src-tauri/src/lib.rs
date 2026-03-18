mod commands;
mod managers;
mod state;

use log::info;
use managers::settings_manager::SettingsManager;
use managers::tray_manager::TrayManager;
use tauri::Manager;

use managers::refresh_manager::RefreshManager;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Load persisted settings (or defaults)
            let config = SettingsManager::load_config(&app_handle);

            // Create shared application state
            let app_state = AppState::new(config.clone());
            let usage_snapshots = app_state.usage_snapshots.clone();
            let app_config = app_state.config.clone();

            // Manage AppState so commands can access it via State<AppState>
            app.manage(app_state);

            // Set up the tray icon with context menu
            TrayManager::setup(&app_handle)?;

            // Start the background refresh manager
            let refresh_mgr = RefreshManager::new(
                usage_snapshots,
                config.refresh_cadence,
                app_handle.clone(),
            );
            refresh_mgr.start(app_config);

            // Hide main window initially (system tray app)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            info!("AIBar setup complete");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_version,
            commands::get_all_providers,
            commands::usage_commands::get_all_usage,
            commands::usage_commands::refresh_provider,
            commands::usage_commands::refresh_all,
            commands::settings_commands::get_settings,
            commands::settings_commands::update_settings,
            commands::settings_commands::toggle_provider,
            commands::credential_commands::store_api_token,
            commands::credential_commands::delete_credential,
            commands::credential_commands::get_credential_status,
            commands::system_commands::open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AIBar");
}
