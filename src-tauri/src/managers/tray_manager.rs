use crate::managers::icon_renderer;
use crate::managers::window_manager;
use log::{error, info};
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

const ICON_SIZE: u32 = 32;

/// Manages the system tray icon and its context menu.
pub struct TrayManager;

impl TrayManager {
    /// Build and register the tray icon with context menu and event handlers.
    pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let refresh_item = MenuItemBuilder::with_id("refresh_all", "Refresh All").build(app)?;
        let settings_item = MenuItemBuilder::with_id("settings", "Settings...").build(app)?;
        let quit_item = MenuItemBuilder::with_id("quit", "Quit AIBar").build(app)?;

        let menu = MenuBuilder::new(app)
            .item(&refresh_item)
            .separator()
            .item(&settings_item)
            .separator()
            .item(&quit_item)
            .build()?;

        // Render default icon
        let icon_data = icon_renderer::render_default_icon();
        let icon = Image::new_owned(icon_data, ICON_SIZE, ICON_SIZE);

        let app_handle = app.clone();
        let app_handle_menu = app.clone();

        let _tray = TrayIconBuilder::new()
            .icon(icon)
            .tooltip("AIBar - AI Usage Monitor")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_tray_icon_event(move |_tray_icon, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } = event
                {
                    if let Err(e) = window_manager::toggle_popup(&app_handle) {
                        error!("TrayManager: failed to toggle popup: {}", e);
                    }
                }
            })
            .on_menu_event(move |app, event| {
                match event.id().as_ref() {
                    "refresh_all" => {
                        info!("TrayManager: refresh all requested from menu");
                        let handle = app_handle_menu.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = handle.state::<crate::state::AppState>();
                            let providers = {
                                let config = state.config.read().await;
                                config.enabled_providers.clone()
                            };
                            crate::managers::refresh_manager::RefreshManager::refresh_all(
                                &providers,
                                &state.usage_snapshots,
                                &handle,
                            )
                            .await;
                        });
                    }
                    "settings" => {
                        info!("TrayManager: settings requested from menu");
                        if let Err(e) = window_manager::open_settings(app) {
                            error!("TrayManager: failed to open settings: {}", e);
                        }
                    }
                    "quit" => {
                        info!("TrayManager: quit requested from menu");
                        app.exit(0);
                    }
                    _ => {}
                }
            })
            .build(app)?;

        Ok(())
    }

    /// Update the tray icon with new usage percentages.
    pub fn update_icon(
        app: &AppHandle,
        primary_pct: f64,
        secondary_pct: f64,
    ) -> Result<(), String> {
        let icon_data = icon_renderer::render_tray_icon(primary_pct, secondary_pct);
        let icon = Image::new_owned(icon_data, ICON_SIZE, ICON_SIZE);

        // Update the tray icon
        if let Some(tray) = app.tray_by_id("main") {
            tray.set_icon(Some(icon))
                .map_err(|e| format!("Failed to set tray icon: {}", e))?;
        }

        Ok(())
    }
}
