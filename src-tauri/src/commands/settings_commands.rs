use tauri::State;

use crate::application::application_state::ApplicationState;
use crate::configuration::server_configuration::ServerConfiguration;
use crate::domain::errors::CommandErrorPayload;
use crate::domain::settings::ApplicationSettingEntry;

#[tauri::command]
pub async fn save_server_configuration(
    state: State<'_, ApplicationState>,
    configuration: ServerConfiguration,
) -> Result<ServerConfiguration, CommandErrorPayload> {
    let saved = state
        .services
        .settings
        .save_server_configuration(configuration)
        .map_err(|error| error.to_command_payload())?;

    if let Ok(mut guard) = state.server_configuration.write() {
        *guard = saved.clone();
    }

    state
        .capture_servers
        .start_or_restart(
            &saved,
            state.services.clone(),
            state.event_publisher.clone(),
        )
        .await
        .map_err(|error| error.to_command_payload())?;

    Ok(saved)
}

#[tauri::command]
pub fn get_application_setting(
    state: State<'_, ApplicationState>,
    key: String,
) -> Result<Option<ApplicationSettingEntry>, CommandErrorPayload> {
    state
        .services
        .settings
        .get_setting(&key)
        .map_err(|error| error.to_command_payload())
}

use crate::services::settings_service::RETENTION_DAYS_SETTING_KEY;

#[tauri::command]
pub fn set_application_setting(
    state: State<'_, ApplicationState>,
    key: String,
    value: serde_json::Value,
) -> Result<ApplicationSettingEntry, CommandErrorPayload> {
    let saved = state
        .services
        .settings
        .set_setting(&key, value)
        .map_err(|error| error.to_command_payload())?;

    if saved.key == RETENTION_DAYS_SETTING_KEY
        && let Some(retention_days) = saved.value.as_u64()
        && let Err(error) = state
            .services
            .notifications
            .prune_expired_by_retention_days(retention_days)
    {
        tracing::warn!("Retention cleanup after settings update failed: {error}");
    }

    Ok(saved)
}
