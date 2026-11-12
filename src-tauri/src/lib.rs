pub mod application;
pub mod commands;
pub mod configuration;
pub mod domain;
pub mod events;
pub mod infrastructure;
pub mod services;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tracing_subscriber::EnvFilter;

use crate::application::application_state::ApplicationState;
use crate::configuration::application_configuration::ApplicationConfiguration;
use crate::events::TauriEventPublisher;
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::security::secret_key_provider::DesktopSecretKeyProvider;
use crate::services::settings_service::SettingsService;

/// Resolves the SQLite database directory inside the platform-standard application data directory.
fn resolve_data_directory(application_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_directory = application_handle
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve application data directory: {error}"))?;

    std::fs::create_dir_all(&data_directory)
        .map_err(|error| format!("Failed to create application data directory: {error}"))?;

    Ok(data_directory)
}

/// Configures and launches the Tauri desktop application runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("notifytrap_lib=info,tauri=info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let application_result = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|application| {
            let data_directory = resolve_data_directory(application.handle()).map_err(|error| {
                Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>
            })?;

            let database_path = data_directory.join(ApplicationConfiguration::DATABASE_FILENAME);
            let database_manager = DatabaseConnectionManager::open_file(&database_path)
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)?;

            let secret_key_provider = Arc::new(DesktopSecretKeyProvider::new(data_directory));
            let mut configuration = ApplicationConfiguration::default();
            let settings_service = SettingsService::new(database_manager.clone());
            configuration.server = settings_service
                .load_server_configuration_or_default(&configuration.server)
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)?;

            let event_publisher = Arc::new(TauriEventPublisher::new(application.handle().clone()));
            let application_state = ApplicationState::new(
                configuration.clone(),
                database_manager,
                secret_key_provider,
                Some(event_publisher),
            );

            match settings_service.load_retention_days_or_default() {
                Ok(retention_days) => {
                    match application_state
                        .services
                        .notifications
                        .prune_expired_by_retention_days(retention_days)
                    {
                        Ok(pruned_count) if pruned_count > 0 => {
                            tracing::info!(
                                "Pruned {pruned_count} expired notifications older than {retention_days} days"
                            );
                        }
                        Ok(_) => {}
                        Err(error) => {
                            tracing::warn!("Startup retention cleanup failed: {error}");
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!("Unable to read retention setting on startup: {error}");
                }
            }

            let startup_result =
                tauri::async_runtime::block_on(application_state.capture_servers.start_or_restart(
                    &configuration.server,
                    application_state.services.clone(),
                    application_state.event_publisher.clone(),
                ));
            if let Err(error) = startup_result {
                tracing::error!("Unable to start local capture servers on boot: {error}");
            }

            application.manage(application_state);

            if let Err(error) = setup_system_tray(application.handle()) {
                tracing::warn!("Unable to initialize system tray icon: {error}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<ApplicationState>();
                let should_minimize = state
                    .services
                    .settings
                    .get_setting("desktop.minimizeToTray")
                    .ok()
                    .flatten()
                    .is_some_and(|entry| entry.value == "true");
                if should_minimize {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::system_commands::get_application_health,
            commands::system_commands::get_server_configuration,
            commands::system_commands::sync_unread_badge,
            commands::system_commands::send_native_notification,
            commands::workspace_commands::list_workspaces,
            commands::workspace_commands::create_workspace,
            commands::workspace_commands::update_workspace,
            commands::workspace_commands::delete_workspace,
            commands::project_commands::list_projects,
            commands::project_commands::create_project,
            commands::project_commands::update_project,
            commands::project_commands::delete_project,
            commands::environment_commands::list_environments,
            commands::environment_commands::create_environment,
            commands::environment_commands::set_default_environment,
            commands::channel_commands::list_channels,
            commands::channel_commands::create_channel,
            commands::channel_commands::set_channel_enabled,
            commands::channel_commands::update_channel_rules,
            commands::channel_commands::reveal_channel_credential,
            commands::channel_commands::rotate_channel_credential,
            commands::channel_commands::delete_channel,
            commands::notification_commands::query_notifications,
            commands::notification_commands::get_notification_detail,
            commands::notification_commands::get_notification_attachment,
            commands::notification_commands::replay_webhook_notification,
            commands::notification_commands::update_notification_status,
            commands::notification_commands::delete_notification,
            commands::notification_commands::clear_scope_notifications,
            commands::settings_commands::save_server_configuration,
            commands::settings_commands::get_application_setting,
            commands::settings_commands::set_application_setting,
        ])
        .build(tauri::generate_context!());

    match application_result {
        Ok(application) => {
            application.run(|application_handle, run_event| {
                if let tauri::RunEvent::Exit = run_event {
                    let state = application_handle.state::<ApplicationState>();
                    tauri::async_runtime::block_on(
                        state
                            .capture_servers
                            .shutdown_all(Some(&*state.event_publisher)),
                    );
                }
            });
        }
        Err(error) => {
            tracing::error!("Fatal Tauri runtime build error: {error}");
        }
    }
}

fn setup_system_tray(application_handle: &tauri::AppHandle) -> Result<(), String> {
    let show_item = MenuItem::with_id(
        application_handle,
        "tray-show",
        "Show NotifyTrap",
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let quit_item = MenuItem::with_id(
        application_handle,
        "tray-quit",
        "Quit NotifyTrap",
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let tray_menu = Menu::with_items(application_handle, &[&show_item, &quit_item])
        .map_err(|error| error.to_string())?;

    let mut builder = TrayIconBuilder::with_id("notifytrap-tray")
        .menu(&tray_menu)
        .tooltip("NotifyTrap")
        .show_menu_on_left_click(false)
        .on_menu_event(|app_handle, event| match event.id.as_ref() {
            "tray-show" => focus_main_window(app_handle),
            "tray-quit" => app_handle.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                focus_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = application_handle.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder
        .build(application_handle)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn focus_main_window(application_handle: &tauri::AppHandle) {
    if let Some(window) = application_handle.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
