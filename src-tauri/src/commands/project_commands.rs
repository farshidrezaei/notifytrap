use tauri::State;

use crate::application::application_state::ApplicationState;
use crate::domain::errors::CommandErrorPayload;
use crate::domain::project::{CreateProjectInput, Project, UpdateProjectInput};

#[tauri::command]
pub fn list_projects(
    state: State<'_, ApplicationState>,
    workspace_id: String,
) -> Result<Vec<Project>, CommandErrorPayload> {
    state
        .services
        .projects
        .list_projects_by_workspace(&workspace_id)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn create_project(
    state: State<'_, ApplicationState>,
    input: CreateProjectInput,
) -> Result<Project, CommandErrorPayload> {
    state
        .services
        .projects
        .create_project(input)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn update_project(
    state: State<'_, ApplicationState>,
    input: UpdateProjectInput,
) -> Result<Project, CommandErrorPayload> {
    state
        .services
        .projects
        .update_project(input)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn delete_project(
    state: State<'_, ApplicationState>,
    project_id: String,
) -> Result<(), CommandErrorPayload> {
    state
        .services
        .projects
        .delete_project(&project_id)
        .map_err(|error| error.to_command_payload())
}
