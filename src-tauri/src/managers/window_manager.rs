use log::info;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// Toggle the main popup window visibility.
pub fn toggle_popup(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window
            .is_visible()
            .map_err(|e| format!("Failed to check window visibility: {}", e))?;

        if visible {
            window
                .hide()
                .map_err(|e| format!("Failed to hide window: {}", e))?;
            info!("WindowManager: popup hidden");
        } else {
            show_popup_near_tray(app)?;
        }
    }
    Ok(())
}

/// Position and show the popup window near the system tray area.
///
/// On most platforms the tray is at the top-right (macOS) or bottom-right
/// (Windows/Linux). We attempt to position the popup window near the tray.
/// Since Tauri v2 does not expose the tray icon position directly, we use
/// cursor position or fall back to a sensible default.
pub fn show_popup_near_tray(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    // Try to get the monitor where the window currently is (or primary monitor)
    let monitor = window
        .current_monitor()
        .map_err(|e| format!("Failed to get monitor: {}", e))?;

    if let Some(monitor) = monitor {
        let monitor_size = monitor.size();
        let monitor_pos = monitor.position();
        let scale = monitor.scale_factor();

        let win_size = window
            .outer_size()
            .map_err(|e| format!("Failed to get window size: {}", e))?;

        // Position at bottom-right of screen, above the taskbar area.
        // Adjust for different platforms.
        let taskbar_height = 48.0_f64; // approximate
        let margin = 8.0_f64;

        let x = (monitor_pos.x as f64
            + (monitor_size.width as f64 / scale)
            - (win_size.width as f64 / scale)
            - margin) as i32;

        let y = (monitor_pos.y as f64
            + (monitor_size.height as f64 / scale)
            - (win_size.height as f64 / scale)
            - taskbar_height
            - margin) as i32;

        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
            x as f64, y as f64,
        )));
    }

    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;
    window
        .set_focus()
        .map_err(|e| format!("Failed to focus window: {}", e))?;

    info!("WindowManager: popup shown");
    Ok(())
}

/// Create and show the settings window.
pub fn open_settings(app: &AppHandle) -> Result<(), String> {
    // Check if settings window already exists
    if let Some(window) = app.get_webview_window("settings") {
        window
            .show()
            .map_err(|e| format!("Failed to show settings: {}", e))?;
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus settings: {}", e))?;
        return Ok(());
    }

    // Create a new settings window
    let _settings_window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("/settings".into()))
        .title("AIBar Settings")
        .inner_size(600.0, 500.0)
        .min_inner_size(500.0, 400.0)
        .resizable(true)
        .center()
        .build()
        .map_err(|e| format!("Failed to create settings window: {}", e))?;

    info!("WindowManager: settings window opened");
    Ok(())
}
