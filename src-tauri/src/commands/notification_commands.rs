use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tauri::State;

use crate::application::application_state::ApplicationState;
use crate::domain::errors::CommandErrorPayload;
use crate::domain::notification::{
    Notification, NotificationAttachmentContent, NotificationDetail, NotificationPage,
    NotificationQuery, NotificationStatus, WebhookReplayInput, WebhookReplayResult,
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
pub fn get_notification_attachment(
    state: State<'_, ApplicationState>,
    attachment_id: String,
) -> Result<NotificationAttachmentContent, CommandErrorPayload> {
    let attachment = state
        .services
        .notifications
        .get_attachment(&attachment_id)
        .map_err(|error| error.to_command_payload())?;

    Ok(NotificationAttachmentContent {
        summary: attachment.summary,
        content_base64: STANDARD.encode(&attachment.content_blob),
    })
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
pub async fn replay_webhook_notification(
    state: State<'_, ApplicationState>,
    input: WebhookReplayInput,
) -> Result<WebhookReplayResult, CommandErrorPayload> {
    state
        .services
        .notifications
        .replay_webhook_notification(input)
        .await
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
