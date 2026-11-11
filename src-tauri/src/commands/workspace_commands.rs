use tauri::State;

use crate::application::application_state::ApplicationState;
use crate::domain::errors::CommandErrorPayload;
use crate::domain::workspace::{CreateWorkspaceInput, UpdateWorkspaceInput, Workspace};

#[tauri::command]
pub fn list_workspaces(
    state: State<'_, ApplicationState>,
) -> Result<Vec<Workspace>, CommandErrorPayload> {
    state
        .services
        .workspaces
        .list_workspaces()
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn create_workspace(
    state: State<'_, ApplicationState>,
    input: CreateWorkspaceInput,
) -> Result<Workspace, CommandErrorPayload> {
    state
        .services
        .workspaces
        .create_workspace(input)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn update_workspace(
    state: State<'_, ApplicationState>,
    input: UpdateWorkspaceInput,
) -> Result<Workspace, CommandErrorPayload> {
    state
        .services
        .workspaces
        .update_workspace(input)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn delete_workspace(
    state: State<'_, ApplicationState>,
    workspace_id: String,
) -> Result<(), CommandErrorPayload> {
    state
        .services
        .workspaces
        .delete_workspace(&workspace_id)
        .map_err(|error| error.to_command_payload())
}
