use serde::Serialize;
use tauri::{Manager, State};
use tauri_plugin_notification::NotificationExt;

use crate::application::application_state::ApplicationState;
use crate::configuration::server_configuration::ServerConfiguration;
use crate::domain::errors::CommandErrorPayload;
use crate::infrastructure::migrations;

/// Runtime health snapshot returned to the desktop frontend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationHealthSnapshot {
    pub application_name: String,
    pub application_version: String,
    pub is_database_connected: bool,
    pub schema_migration_version: i64,
    pub is_http_server_running: bool,
    pub is_smtp_server_running: bool,
    pub http_endpoint: String,
    pub smtp_endpoint: String,
}

/// Returns current database connectivity, migration version, and local server status.
#[tauri::command]
pub fn get_application_health(
    state: State<'_, ApplicationState>,
) -> Result<ApplicationHealthSnapshot, CommandErrorPayload> {
    let migration_version = state
        .database_manager
        .with_connection(|connection| migrations::current_migration_version(connection))
        .map_err(|error| error.to_command_payload())?;

    let server_status = state.capture_servers.current_status();

    Ok(ApplicationHealthSnapshot {
        application_name: state.application_name.clone(),
        application_version: state.application_version.clone(),
        is_database_connected: true,
        schema_migration_version: migration_version,
        is_http_server_running: server_status.is_http_server_running,
        is_smtp_server_running: server_status.is_smtp_server_running,
        http_endpoint: server_status.http_endpoint,
        smtp_endpoint: server_status.smtp_endpoint,
    })
}

/// Returns current server configuration for display in Settings and Overview.
#[tauri::command]
pub fn get_server_configuration(
    state: State<'_, ApplicationState>,
) -> Result<ServerConfiguration, CommandErrorPayload> {
    Ok(state.current_server_configuration())
}

/// Synchronizes unread notification count onto the taskbar/dock badge and system tray icon.
#[tauri::command]
pub fn sync_unread_badge(application_handle: tauri::AppHandle, unread_count: u64) {
    if let Some(window) = application_handle.get_webview_window("main") {
        let badge_value = if unread_count > 0 {
            Some(i64::try_from(unread_count).unwrap_or(99))
        } else {
            None
        };
        let _ = window.set_badge_count(badge_value);
    }

    if let Some(tray) = application_handle.tray_by_id("notifytrap-tray") {
        let tooltip = if unread_count > 0 {
            format!("NotifyTrap ({unread_count} unread)")
        } else {
            "NotifyTrap".to_string()
        };
        let _ = tray.set_tooltip(Some(&tooltip));

        #[cfg(target_os = "macos")]
        {
            let title = if unread_count > 0 {
                Some(unread_count.to_string())
            } else {
                None
            };
            let _ = tray.set_title(title.as_deref());
        }

        if let Some(base_icon) = application_handle.default_window_icon() {
            if unread_count == 0 {
                let _ = tray.set_icon(Some(base_icon.clone()));
            } else {
                let badged_icon = build_badged_tray_icon(base_icon);
                let _ = tray.set_icon(Some(badged_icon));
            }
        }
    }
}

/// Dispatches a native OS desktop notification via `tauri-plugin-notification`.
#[tauri::command]
pub fn send_native_notification(
    application_handle: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), CommandErrorPayload> {
    let trimmed_title = title.trim();
    let effective_title = if trimmed_title.is_empty() {
        "NotifyTrap"
    } else {
        trimmed_title
    };
    application_handle
        .notification()
        .builder()
        .title(effective_title)
        .body(body.trim())
        .show()
        .map_err(|error| CommandErrorPayload {
            code: "NOTIFICATION_ERROR".to_string(),
            message: format!("Failed to display native notification: {error}"),
        })
}

fn build_badged_tray_icon(base_icon: &tauri::image::Image<'_>) -> tauri::image::Image<'static> {
    let width = base_icon.width();
    let height = base_icon.height();
    let mut rgba = base_icon.rgba().to_vec();

    if width >= 16 && height >= 16 {
        let radius = (width.min(height) as f32 * 0.22).max(4.0);
        let center_x = width as f32 - radius - 1.0;
        let center_y = radius + 1.0;
        let outer_radius_sq = (radius + 1.2) * (radius + 1.2);
        let inner_radius_sq = radius * radius;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist_sq = dx * dx + dy * dy;
                let offset = ((y * width + x) * 4) as usize;
                if offset + 3 >= rgba.len() {
                    continue;
                }
                if dist_sq <= inner_radius_sq {
                    rgba[offset] = 239;
                    rgba[offset + 1] = 68;
                    rgba[offset + 2] = 68;
                    rgba[offset + 3] = 255;
                } else if dist_sq <= outer_radius_sq {
                    rgba[offset] = 255;
                    rgba[offset + 1] = 255;
                    rgba[offset + 2] = 255;
                    rgba[offset + 3] = 255;
                }
            }
        }
    }

    tauri::image::Image::new_owned(rgba, width, height)
}
