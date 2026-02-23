mod commands;
mod state;
mod terminal;

use commands::{
    ai::stream_ai_response,
    hotkey::register_hotkey,
    keychain::{delete_api_key, get_api_key, save_api_key},
    paste::{paste_to_terminal, confirm_terminal_command},
    permissions::{check_accessibility_permission, open_accessibility_settings},
    safety::{check_destructive, get_destructive_explanation},
    terminal::{get_app_context, get_terminal_context},
    tray::setup_tray,
    window::{hide_overlay, show_overlay},
    xai::validate_and_fetch_models,
};
use state::AppState;
use tauri::Manager;
use tauri_nspanel::{
    CollectionBehavior, PanelLevel, StyleMask, WebviewWindowExt,
};
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

// Define the custom NSPanel subclass with can_become_key_window = true
// This allows the panel to accept keyboard input (for the command input field)
// while still being an NSPanel (auxiliary window that doesn't steal focus permanently)
tauri_nspanel::tauri_panel! {
    OverlayPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        // NSPanel plugin for floating overlay above fullscreen apps
        .plugin(tauri_nspanel::init())
        // Global hotkey plugin for system-wide Cmd+K trigger
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Positioner plugin for predefined window placement helpers
        .plugin(tauri_plugin_positioner::init())
        // Store plugin for persistent config (hotkey preference, API key, etc.)
        .plugin(tauri_plugin_store::Builder::default().build())
        // HTTP plugin for Rust-side xAI API calls (bearer token stays off the JS layer)
        .plugin(tauri_plugin_http::init())
        // Shared application state
        .manage(AppState::default())
        .setup(|app| {
            // CRITICAL: Set ActivationPolicy::Accessory FIRST before any window operations.
            // This:
            // 1. Hides the Dock icon (required behavior per spec)
            // 2. Prevents Stage Manager rendering glitches on macOS Sonoma (Pitfall 1)
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Get the main webview window for NSPanel conversion
            let window = app
                .get_webview_window("main")
                .expect("Window 'main' should exist per tauri.conf.json");

            // Apply frosted glass vibrancy effect (Spotlight/Raycast aesthetic)
            // HudWindow material gives a dark, solid-feeling translucent panel
            // 12.0 corner radius matches the CSS border-radius
            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(12.0))
                .expect("Failed to apply NSVisualEffectView vibrancy");

            // Convert standard Tauri window to NSPanel for correct overlay behavior:
            // - OverlayPanel has can_become_key_window = true for keyboard input
            // - is_floating_panel = true for proper panel behavior
            // - show_and_make_key() accepts input without permanently stealing focus
            // - Dismissed panel returns focus to the previously active application
            let panel = window
                .to_panel::<OverlayPanel>()
                .expect("Failed to convert window to NSPanel");

            // Set panel level above the menu bar so it floats over fullscreen apps
            // Status level (25) = NSMainMenuWindowLevel + 1
            panel.set_level(PanelLevel::Status.value());

            // Allow the panel to appear alongside fullscreen apps (Raycast-style)
            panel.set_collection_behavior(
                CollectionBehavior::new()
                    .full_screen_auxiliary()
                    .can_join_all_spaces()
                    .into(),
            );

            // Nonactivating panel style: accepts keyboard input but doesn't activate the app
            // This ensures the underlying app isn't deactivated when the overlay appears
            panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

            // Set up menu bar tray icon with K.png branding
            setup_tray(app)?;

            // Register default hotkey "Super+K" (Cmd+K on macOS)
            // If registration fails (hotkey conflict), the error is logged but app continues
            // The user can change the hotkey via "Change Hotkey..." in the tray menu
            let app_handle = app.handle().clone();
            if let Err(e) = register_hotkey(app_handle, "Super+K".to_string()) {
                eprintln!(
                    "Warning: Failed to register default hotkey 'Super+K': {}. \
                    Use 'Change Hotkey...' in the menu bar to set a different hotkey.",
                    e
                );
            }

            // Window is configured as `visible: false` in tauri.conf.json,
            // but ensure it's hidden in case the config is overridden
            window.hide().ok();

            Ok(())
        })
        // Expose IPC commands to the frontend
        .invoke_handler(tauri::generate_handler![
            show_overlay,
            hide_overlay,
            register_hotkey,
            save_api_key,
            get_api_key,
            delete_api_key,
            validate_and_fetch_models,
            open_accessibility_settings,
            check_accessibility_permission,
            get_terminal_context,
            get_app_context,
            stream_ai_response,
            check_destructive,
            get_destructive_explanation,
            paste_to_terminal,
            confirm_terminal_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CMD+K application");
}
