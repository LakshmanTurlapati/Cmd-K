use tauri::{AppHandle, Emitter, LogicalPosition, Manager};
use tauri_nspanel::ManagerExt;

use crate::state::AppState;

/// Show the overlay panel and position it at 25% down from the top of the
/// current monitor, centered horizontally.
///
/// Uses `current_monitor()` instead of `primary_monitor()` so the overlay
/// appears on the display where the user is currently working (multi-monitor support).
///
/// Physical pixels are divided by `scale_factor()` before computing logical
/// coordinates -- required for correct placement on Retina displays (Pitfall 5).
#[tauri::command]
pub fn show_overlay(app: AppHandle) -> Result<(), String> {
    // Position overlay on the monitor where the user is working
    position_overlay(&app)?;

    let panel = app
        .get_webview_panel("main")
        .map_err(|e| format!("Panel 'main' not found: {:?}", e))?;

    panel.show_and_make_key();

    // Update visibility state
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut visible) = state.overlay_visible.lock() {
            *visible = true;
        }
    }

    // Notify frontend to auto-focus input
    let _ = app.emit("overlay-shown", ());

    Ok(())
}

/// Hide the overlay panel by calling hide() to remove it from screen.
#[tauri::command]
pub fn hide_overlay(app: AppHandle) -> Result<(), String> {
    let panel = app
        .get_webview_panel("main")
        .map_err(|e| format!("Panel 'main' not found: {:?}", e))?;

    panel.hide();

    // Update visibility state
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut visible) = state.overlay_visible.lock() {
            *visible = false;
        }
    }

    Ok(())
}

/// Toggle the overlay: show if hidden, hide if visible.
/// Called by the global hotkey handler.
pub fn toggle_overlay(app: &AppHandle) {
    let is_visible = app
        .try_state::<AppState>()
        .and_then(|state| state.overlay_visible.lock().ok().map(|v| *v))
        .unwrap_or(false);

    if is_visible {
        let _ = hide_overlay(app.clone());
    } else {
        let _ = show_overlay(app.clone());
    }
}

/// Compute and set the overlay window position:
/// - Centered horizontally on the current monitor
/// - 25% down from the top of the current monitor
///
/// Converts physical pixel dimensions to logical (point) coordinates using
/// `scale_factor()` to handle Retina displays correctly.
fn position_overlay(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Window 'main' not found".to_string())?;

    // Use current_monitor() for multi-monitor support (user works on current display)
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;

    let scale = monitor.scale_factor();
    let size = monitor.size();
    let monitor_pos = monitor.position();

    // Convert physical pixels to logical coordinates (Pitfall 5 fix)
    let logical_w = size.width as f64 / scale;
    let logical_h = size.height as f64 / scale;
    let monitor_x = monitor_pos.x as f64 / scale;
    let monitor_y = monitor_pos.y as f64 / scale;

    // Overlay is 640px wide (matching Cursor Cmd+K reference)
    let overlay_w = 640.0_f64;
    let overlay_x = monitor_x + (logical_w - overlay_w) / 2.0;
    let overlay_y = monitor_y + logical_h * 0.25;

    window
        .set_position(LogicalPosition::new(overlay_x, overlay_y))
        .map_err(|e| e.to_string())?;

    Ok(())
}
