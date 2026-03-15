use crate::state::{UpdateState, UpdateStatus};
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tauri_plugin_updater::UpdaterExt;
use tokio::time::{interval, Duration};

/// Create a new UpdateState with default (idle) values.
pub fn create_update_state() -> UpdateState {
    UpdateState {
        status: std::sync::Mutex::new(UpdateStatus::Idle),
        pending_update: std::sync::Mutex::new(None),
        pending_bytes: std::sync::Mutex::new(None),
        menu_item: std::sync::Mutex::new(None),
    }
}

/// Spawn the background update checker. Runs an immediate check on launch,
/// then re-checks every 24 hours.
pub fn spawn_update_checker(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Initial check on launch
        check_and_download(&app_handle).await;

        // Re-check every 24 hours
        let mut timer = interval(Duration::from_secs(86400));
        timer.tick().await; // skip first immediate tick (already checked above)
        loop {
            timer.tick().await;
            check_and_download(&app_handle).await;
        }
    });
}

/// Check for updates and auto-download if available.
/// Silent on all errors (per CONTEXT.md: retry next cycle).
async fn check_and_download(app: &tauri::AppHandle) {
    // Read auto-update preference from settings store
    let auto_update = app
        .store("settings.json")
        .ok()
        .and_then(|s| s.get("autoUpdate"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // default: enabled

    if !auto_update {
        if let Ok(mut status) = app.state::<UpdateState>().status.lock() {
            *status = UpdateStatus::Disabled;
        }
        update_tray_text(app, &UpdateStatus::Disabled);
        return;
    }

    // Set status to Checking
    {
        if let Ok(mut status) = app.state::<UpdateState>().status.lock() {
            *status = UpdateStatus::Checking;
        }
        update_tray_text(app, &UpdateStatus::Checking);
    }

    // Check for updates
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("[updater] Failed to get updater: {}", e);
            set_idle(app);
            return;
        }
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            // No update available
            set_idle(app);
            return;
        }
        Err(e) => {
            eprintln!("[updater] Check failed (silent): {}", e);
            set_idle(app);
            return;
        }
    };

    let version = update.version.clone();

    // Set status to Available
    {
        if let Ok(mut status) = app.state::<UpdateState>().status.lock() {
            *status = UpdateStatus::Available(version.clone());
        }
        update_tray_text(app, &UpdateStatus::Available(version.clone()));
    }

    // Set status to Downloading
    {
        if let Ok(mut status) = app.state::<UpdateState>().status.lock() {
            *status = UpdateStatus::Downloading(version.clone());
        }
        update_tray_text(app, &UpdateStatus::Downloading(version.clone()));
    }

    // Auto-download
    match update.download(|_, _| {}, || {}).await {
        Ok(bytes) => {
            // Store bytes and update for install-on-quit
            let state = app.state::<UpdateState>();
            if let Ok(mut pb) = state.pending_bytes.lock() {
                *pb = Some(bytes);
            }
            if let Ok(mut pu) = state.pending_update.lock() {
                *pu = Some(update);
            }
            if let Ok(mut s) = state.status.lock() {
                *s = UpdateStatus::Ready(version.clone());
            }
            update_tray_text(app, &UpdateStatus::Ready(version));
        }
        Err(e) => {
            eprintln!("[updater] Download failed (silent): {}", e);
            set_idle(app);
        }
    }
}

/// Update the tray menu item text to reflect current update status.
pub fn update_tray_text(app: &tauri::AppHandle, status: &UpdateStatus) {
    let state = match app.try_state::<UpdateState>() {
        Some(s) => s,
        None => return,
    };

    let menu_item = match state.menu_item.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let item = match menu_item.as_ref() {
        Some(i) => i,
        None => return,
    };

    let text = match status {
        UpdateStatus::Idle => "Check for Updates...".to_string(),
        UpdateStatus::Checking => "Checking for Updates...".to_string(),
        UpdateStatus::Available(v) => format!("Update Available (v{})", v),
        UpdateStatus::Downloading(v) => format!("Downloading v{}...", v),
        UpdateStatus::Ready(v) => format!("Update Ready v{} (restart to apply)", v),
        UpdateStatus::Disabled => "Check for Updates...".to_string(),
    };

    let _ = item.set_text(text);
}

/// Install the pending update (called from quit handler).
/// On Windows, this triggers the NSIS passive installer which force-exits.
/// On macOS, this replaces the .app bundle.
/// On Linux, this replaces the AppImage file (skips if location is not writable).
pub fn install_pending_update(app: &tauri::AppHandle) {
    let state = match app.try_state::<UpdateState>() {
        Some(s) => s,
        None => return,
    };

    let bytes = state.pending_bytes.lock().ok().and_then(|mut b| b.take());
    let update = state.pending_update.lock().ok().and_then(|mut u| u.take());

    if let (Some(update), Some(bytes)) = (update, bytes) {
        // On Linux (AppImage), check if the executable location is writable
        // before attempting install. If not writable, warn and skip.
        #[cfg(target_os = "linux")]
        {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    let test_path = parent.join(".cmd-k-update-test");
                    match std::fs::File::create(&test_path) {
                        Ok(_) => {
                            let _ = std::fs::remove_file(&test_path);
                        }
                        Err(e) => {
                            eprintln!(
                                "[updater] AppImage location not writable ({}): {}. Skipping update.",
                                parent.display(),
                                e
                            );
                            // Update tray to inform user
                            if let Ok(mut s) = app.state::<UpdateState>().status.lock() {
                                *s = UpdateStatus::Idle;
                            }
                            update_tray_text(app, &UpdateStatus::Idle);
                            return;
                        }
                    }
                }
            }
        }

        if let Err(e) = update.install(&bytes) {
            eprintln!("[updater] Install failed: {}", e);
        }
    }
}

/// Manual update check triggered by user clicking "Check for Updates..." menu item.
pub async fn manual_update_check(app: tauri::AppHandle) {
    check_and_download(&app).await;
}

/// Helper to reset status to Idle and update tray text.
fn set_idle(app: &tauri::AppHandle) {
    if let Ok(mut status) = app.state::<UpdateState>().status.lock() {
        *status = UpdateStatus::Idle;
    }
    update_tray_text(app, &UpdateStatus::Idle);
}
