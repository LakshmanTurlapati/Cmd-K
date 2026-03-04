mod commands;
mod state;
mod terminal;

use commands::{
    ai::stream_ai_response,
    history::{get_window_key, get_window_history, add_history_entry, clear_all_history},
    hotkey::register_hotkey,
    keychain::{delete_api_key, get_api_key, save_api_key},
    paste::{paste_to_terminal, confirm_terminal_command},
    permissions::{check_accessibility_permission, open_accessibility_settings, open_url, request_accessibility_permission},
    safety::{check_destructive, get_destructive_explanation},
    terminal::{get_app_context, get_terminal_context},
    tray::setup_tray,
    window::{hide_overlay, show_overlay, set_overlay_position},
    xai::validate_and_fetch_models,
};
use state::AppState;
use tauri::Manager;

#[cfg(target_os = "macos")]
use tauri_nspanel::{
    CollectionBehavior, PanelLevel, StyleMask, WebviewWindowExt,
};
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

// Define the custom NSPanel subclass with can_become_key_window = true
// This allows the panel to accept keyboard input (for the command input field)
// while still being an NSPanel (auxiliary window that doesn't steal focus permanently)
#[cfg(target_os = "macos")]
tauri_nspanel::tauri_panel! {
    OverlayPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
    }
}

pub fn run() {
    let mut builder = tauri::Builder::default();

    // NSPanel plugin for floating overlay above fullscreen apps (macOS only)
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
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

            // macOS: Convert standard Tauri window to NSPanel for correct overlay behavior
            #[cfg(target_os = "macos")]
            {
                // Convert standard Tauri window to NSPanel for correct overlay behavior:
                // - OverlayPanel has can_become_key_window = true for keyboard input
                // - is_floating_panel = true for proper panel behavior
                // - show_and_make_key() accepts input without permanently stealing focus
                // - Dismissed panel returns focus to the previously active application
                let panel = window
                    .to_panel::<OverlayPanel>()
                    .expect("Failed to convert window to NSPanel");

                // Set panel level to Floating (3) — above normal app windows but below
                // system UI (permission dialogs, Notification Center, Spotlight).
                // Floating (3) > Normal (0), so the overlay stays above all standard apps.
                // System overlays use higher levels (ModalPanel=8, MainMenu=24, Status=25)
                // and will correctly render above this panel.
                // Combined with full_screen_auxiliary() collection behavior, the overlay
                // still appears above fullscreen apps.
                panel.set_level(PanelLevel::Floating.value());

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

                // Disable the macOS window-level shadow. The rectangular NSPanel shadow
                // does not follow the vibrancy corner radius, producing a squared-off
                // shadow at the bottom. The CSS shadow-2xl on the overlay div provides
                // a properly rounded shadow instead.
                panel.set_has_shadow(false);
            }

            // Windows: Acrylic vibrancy, always-on-top, WS_EX_TOOLWINDOW
            //
            // DPI awareness: Tauri v2 + WebView2 handles DPI scaling automatically.
            // Window uses logical coordinates (LogicalPosition/LogicalSize) for correct
            // rendering at 100%, 150%, 200% scaling. No manual DPI configuration needed.
            #[cfg(target_os = "windows")]
            {
                use window_vibrancy::apply_acrylic;

                // Apply Acrylic blur -- closest to macOS NSVisualEffectView behind-window blur
                // RGBA: (18, 18, 18, 125) matches macOS HudWindow darkness
                apply_acrylic(&window, Some((18, 18, 18, 125)))
                    .expect("Acrylic vibrancy requires Windows 10 1903+");
                eprintln!("[setup] Acrylic vibrancy applied");

                // Always-on-top: equivalent to macOS NSPanel floating level
                window
                    .set_always_on_top(true)
                    .expect("Failed to set always-on-top");

                // Apply WS_EX_TOOLWINDOW to hide from Alt+Tab and taskbar.
                // Tauri's skipTaskbar is buggy (see issue #10422), so we set
                // the extended window style directly via Win32 API.
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                use windows_sys::Win32::UI::WindowsAndMessaging::*;

                if let Ok(handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                        let hwnd_isize = win32.hwnd.get() as isize;
                        let hwnd = hwnd_isize as windows_sys::Win32::Foundation::HWND;
                        unsafe {
                            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                            let new_style = (ex_style | WS_EX_TOOLWINDOW as isize)
                                & !(WS_EX_APPWINDOW as isize);
                            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
                        }
                        eprintln!("[setup] WS_EX_TOOLWINDOW applied (hwnd={})", hwnd_isize);
                    }
                }
            }

            // Set up menu bar tray icon with K.png branding
            setup_tray(app)?;

            // Register default hotkey -- platform-specific:
            // macOS: Super+K (Cmd+K)
            // Windows: Ctrl+K
            // If registration fails (hotkey conflict), the error is logged but app continues
            // The user can change the hotkey via "Change Hotkey..." in the tray menu
            #[cfg(target_os = "macos")]
            let default_hotkey = "Super+K";
            #[cfg(not(target_os = "macos"))]
            let default_hotkey = "Ctrl+K";

            let app_handle = app.handle().clone();
            if let Err(e) = register_hotkey(app_handle, default_hotkey.to_string()) {
                eprintln!(
                    "Warning: Failed to register default hotkey '{}': {}. \
                    Use 'Change Hotkey...' in the menu bar to set a different hotkey.",
                    default_hotkey, e
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
            request_accessibility_permission,
            get_terminal_context,
            get_app_context,
            stream_ai_response,
            check_destructive,
            get_destructive_explanation,
            paste_to_terminal,
            confirm_terminal_command,
            open_url,
            get_window_key,
            get_window_history,
            add_history_entry,
            clear_all_history,
            set_overlay_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CMD+K application");
}
