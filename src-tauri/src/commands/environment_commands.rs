use tauri::State;

use crate::application::application_state::ApplicationState;
use crate::domain::environment::{CreateEnvironmentInput, Environment};
use crate::domain::errors::CommandErrorPayload;

#[tauri::command]
pub fn list_environments(
    state: State<'_, ApplicationState>,
    project_id: String,
) -> Result<Vec<Environment>, CommandErrorPayload> {
    state
        .services
        .environments
        .list_environments_by_project(&project_id)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn create_environment(
    state: State<'_, ApplicationState>,
    input: CreateEnvironmentInput,
) -> Result<Environment, CommandErrorPayload> {
    state
        .services
        .environments
        .create_environment(input)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn set_default_environment(
    state: State<'_, ApplicationState>,
    project_id: String,
    environment_id: String,
) -> Result<Vec<Environment>, CommandErrorPayload> {
    state
        .services
        .environments
        .set_default_environment(&project_id, &environment_id)
        .map_err(|error| error.to_command_payload())
}
