use tauri::State;

use crate::application::application_state::ApplicationState;
use crate::domain::errors::CommandErrorPayload;
use crate::domain::notification::{
    Notification, NotificationDetail, NotificationPage, NotificationQuery, NotificationStatus,
};

#[tauri::command]
pub fn query_notifications(
    state: State<'_, ApplicationState>,
    query: NotificationQuery,
) -> Result<NotificationPage, CommandErrorPayload> {
    state
        .services
        .notifications
        .query_notifications(query)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn get_notification_detail(
    state: State<'_, ApplicationState>,
    notification_id: String,
) -> Result<NotificationDetail, CommandErrorPayload> {
    state
        .services
        .notifications
        .get_notification_detail(&notification_id)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn update_notification_status(
    state: State<'_, ApplicationState>,
    notification_id: String,
    status: NotificationStatus,
) -> Result<Notification, CommandErrorPayload> {
    state
        .services
        .notifications
        .update_notification_status(&notification_id, status)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn delete_notification(
    state: State<'_, ApplicationState>,
    notification_id: String,
) -> Result<(), CommandErrorPayload> {
    state
        .services
        .notifications
        .delete_notification(&notification_id)
        .map_err(|error| error.to_command_payload())
}

#[tauri::command]
pub fn clear_scope_notifications(
    state: State<'_, ApplicationState>,
    workspace_id: String,
    project_id: String,
    environment_id: String,
) -> Result<u64, CommandErrorPayload> {
    state
        .services
        .notifications
        .clear_scope_notifications(&workspace_id, &project_id, &environment_id)
        .map_err(|error| error.to_command_payload())
}
