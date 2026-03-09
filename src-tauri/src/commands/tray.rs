use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, Emitter, Manager,
};

use super::updater;
use super::window::show_overlay;
use crate::state::UpdateState;

/// Set up the menu bar tray icon with K-white.png branding and the required menu items.
///
/// Menu items (per CONTEXT.md spec):
/// - Settings...
/// - Change Hotkey...
/// - About
/// - [separator]
/// - Quit CMD+K
///
/// On macOS: `show_menu_on_left_click(false)` follows macOS convention (right-click for menu).
/// On Windows: left-click shows menu (Windows convention).
///
/// The K-white.png image is loaded from the repo root (one level up from src-tauri/).
pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let check_for_updates = MenuItem::with_id(
        app,
        "check_for_updates",
        "Check for Updates...",
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let change_hotkey =
        MenuItem::with_id(app, "change_hotkey", "Change Hotkey...", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit CMD+K", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&check_for_updates, &settings, &change_hotkey, &about, &separator, &quit],
    )?;

    // Store the menu item reference in UpdateState for dynamic text updates
    let update_state = app.state::<UpdateState>();
    if let Ok(mut mi) = update_state.menu_item.lock() {
        *mi = Some(check_for_updates);
    }

    // Load K-white.png from repo root as the tray icon
    let icon = load_tray_icon(app);

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "check_for_updates" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    updater::manual_update_check(app_handle).await;
                });
            }
            "settings" => {
                let _ = show_overlay(app.clone());
                let _ = app.emit("open-settings", ());
            }
            "change_hotkey" => {
                let _ = show_overlay(app.clone());
                let _ = app.emit("open-hotkey-config", ());
            }
            "about" => {
                let _ = show_overlay(app.clone());
                let _ = app.emit("open-about", ());
            }
            "quit" => {
                // Install pending update before exiting (if any downloaded)
                updater::install_pending_update(app);
                app.exit(0);
            }
            _ => {}
        });

    // macOS: right-click for menu (macOS convention), template icon (auto dark/light)
    #[cfg(target_os = "macos")]
    {
        builder = builder
            .show_menu_on_left_click(false)
            .icon_as_template(true);
    }

    // Windows: left-click shows menu (Windows convention), non-template icon
    #[cfg(target_os = "windows")]
    {
        builder = builder
            .show_menu_on_left_click(true)
            .icon_as_template(false);
    }

    // Fallback for other platforms: use defaults
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        builder = builder
            .show_menu_on_left_click(true)
            .icon_as_template(false);
    }

    if let Some(icon) = icon {
        builder = builder.icon(icon);
    }

    builder.build(app)?;

    Ok(())
}

/// Attempt to load K-white.png from the repo root as a template icon.
/// Falls back gracefully if the file is not found.
fn load_tray_icon(app: &App) -> Option<Image<'static>> {
    // Try loading K-white.png from the resource directory (bundled apps).
    // Tauri maps the "../K-white.png" resource path to "_up_/K-white.png" inside Resources/.
    let resource_path = app
        .path()
        .resource_dir()
        .ok()
        .map(|p| p.join("_up_").join("K-white.png"));

    if let Some(path) = resource_path {
        if path.exists() {
            if let Ok(img) = Image::from_path(&path) {
                return Some(img);
            }
        }
    }

    // Fallback: try relative path from src-tauri/ directory for dev mode
    let dev_path = std::path::Path::new("../K-white.png");
    if dev_path.exists() {
        if let Ok(img) = Image::from_path(dev_path) {
            return Some(img);
        }
    }

    // Second fallback: current directory K-white.png
    let cwd_path = std::path::Path::new("K-white.png");
    if cwd_path.exists() {
        if let Ok(img) = Image::from_path(cwd_path) {
            return Some(img);
        }
    }

    None
}
