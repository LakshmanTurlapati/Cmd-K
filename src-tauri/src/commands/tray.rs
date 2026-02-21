use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, Emitter, Manager,
};

/// Set up the menu bar tray icon with K.png branding and the required menu items.
///
/// Menu items (per CONTEXT.md spec):
/// - Settings...
/// - Change Hotkey...
/// - About
/// - [separator]
/// - Quit CMD+K
///
/// Uses `show_menu_on_left_click(false)` to follow macOS convention of showing the
/// menu only on right-click or control-click.
///
/// The K.png image is loaded from the repo root (one level up from src-tauri/).
pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let change_hotkey =
        MenuItem::with_id(app, "change_hotkey", "Change Hotkey...", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit CMD+K", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&settings, &change_hotkey, &about, &separator, &quit])?;

    // Load K.png from repo root as the tray icon
    let icon = load_tray_icon(app);

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                let _ = app.emit("open-settings", ());
            }
            "change_hotkey" => {
                let _ = app.emit("open-hotkey-config", ());
            }
            "about" => {
                let _ = app.emit("open-about", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        });

    if let Some(icon) = icon {
        builder = builder.icon(icon);
    }

    builder.build(app)?;

    Ok(())
}

/// Attempt to load K.png from the repo root as a template icon.
/// Falls back gracefully if the file is not found.
fn load_tray_icon(app: &App) -> Option<Image<'static>> {
    // Try loading K.png from the resource directory (bundled apps)
    let resource_path = app
        .path()
        .resource_dir()
        .ok()
        .map(|p| p.join("K.png"));

    if let Some(path) = resource_path {
        if path.exists() {
            if let Ok(img) = Image::from_path(&path) {
                return Some(img);
            }
        }
    }

    // Fallback: try relative path from src-tauri/ directory for dev mode
    let dev_path = std::path::Path::new("../K.png");
    if dev_path.exists() {
        if let Ok(img) = Image::from_path(dev_path) {
            return Some(img);
        }
    }

    // Second fallback: current directory K.png
    let cwd_path = std::path::Path::new("K.png");
    if cwd_path.exists() {
        if let Ok(img) = Image::from_path(cwd_path) {
            return Some(img);
        }
    }

    None
}
