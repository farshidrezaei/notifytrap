use tauri::State;

use crate::application::application_state::ApplicationState;
use crate::domain::channel::{
    ChannelCredentialReveal, ChannelSummary, CreateChannelInput, CreatedChannelResult,
};
use crate::domain::errors::CommandErrorPayload;

#[tauri::command]
pub fn list_channels(
    state: State<'_, ApplicationState>,
    workspace_id: String,
    project_id: String,
    environment_id: String,
) -> Result<Vec<ChannelSummary>, CommandErrorPayload> {
    state
        .services
        .channels
        .list_channels_by_scope(&workspace_id, &project_id, &environment_id)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn create_channel(
    state: State<'_, ApplicationState>,
    input: CreateChannelInput,
) -> Result<CreatedChannelResult, CommandErrorPayload> {
    state
        .services
        .channels
        .create_channel(input)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn set_channel_enabled(
    state: State<'_, ApplicationState>,
    channel_id: String,
    is_enabled: bool,
) -> Result<ChannelSummary, CommandErrorPayload> {
    state
        .services
        .channels
        .set_channel_enabled(&channel_id, is_enabled)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn reveal_channel_credential(
    state: State<'_, ApplicationState>,
    channel_id: String,
) -> Result<ChannelCredentialReveal, CommandErrorPayload> {
    state
        .services
        .channels
        .reveal_channel_credential(&channel_id)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn rotate_channel_credential(
    state: State<'_, ApplicationState>,
    channel_id: String,
) -> Result<ChannelCredentialReveal, CommandErrorPayload> {
    state
        .services
        .channels
        .rotate_channel_credential(&channel_id)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn delete_channel(
    state: State<'_, ApplicationState>,
    channel_id: String,
) -> Result<(), CommandErrorPayload> {
    state
        .services
        .channels
        .delete_channel(&channel_id)
        .map_err(|error| error.to_command_payload())
}
