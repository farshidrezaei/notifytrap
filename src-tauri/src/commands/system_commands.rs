use serde::Serialize;
use tauri::State;

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
