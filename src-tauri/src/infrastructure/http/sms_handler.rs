use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::channel::ChannelType;
use crate::domain::notification::CaptureNotificationInput;
use crate::domain::validation::validate_bounded_text;
use crate::infrastructure::http::capture_auth::{
    PREVIEW_MAX_CHARACTERS, authenticate_header_channel, extract_test_run_identifier,
    persist_and_emit, truncate_preview,
};
use crate::infrastructure::http::capture_error::{CaptureReceiptResponse, HttpCaptureError};
use crate::infrastructure::http::routes::HttpCaptureState;
use crate::infrastructure::http::webhook_handler::apply_channel_chaos_if_enabled;

#[derive(Debug, Deserialize)]
pub struct SmsCapturePayload {
    pub to: String,
    pub message: String,
    pub from: Option<String>,
    pub metadata: Option<Value>,
}

pub async fn handle_sms_capture(
    State(state): State<HttpCaptureState>,
    headers: HeaderMap,
    payload_result: Result<Json<SmsCapturePayload>, axum::extract::rejection::JsonRejection>,
) -> Result<(StatusCode, Json<CaptureReceiptResponse>), HttpCaptureError> {
    let channel = authenticate_header_channel(&headers, &state, ChannelType::Sms)?;
    apply_channel_chaos_if_enabled(&channel.chaos_rule).await?;
    let test_run_identifier = extract_test_run_identifier(&headers)?;
    let Json(payload) = payload_result.map_err(HttpCaptureError::from_json_rejection)?;

    let recipient = validate_bounded_text(&payload.to, "to", 3, 32, false)
        .map_err(|error| HttpCaptureError::bad_request(error.to_string()))?;
    let message_text = validate_bounded_text(&payload.message, "message", 1, 16_000, true)
        .map_err(|error| HttpCaptureError::bad_request(error.to_string()))?;

    let sender = match payload.from.as_deref().map(str::trim) {
        Some(value) if !value.is_empty() => validate_bounded_text(value, "from", 1, 64, false)
            .map_err(|error| HttpCaptureError::bad_request(error.to_string()))?,
        _ => "NotifyTrap".to_string(),
    };

    let canonical_payload = json!({
        "to": recipient,
        "from": sender,
        "message": message_text,
        "metadata": payload.metadata.unwrap_or_else(|| json!({})),
    });

    let raw_payload = serde_json::to_string_pretty(&canonical_payload)
        .unwrap_or_else(|_| canonical_payload.to_string());

    persist_and_emit(
        &state,
        CaptureNotificationInput {
            workspace_id: channel.workspace_id,
            project_id: channel.project_id,
            environment_id: channel.environment_id,
            channel_id: channel.id,
            channel_type: ChannelType::Sms,
            recipient,
            sender,
            subject: format!("SMS • {}", truncate_preview(&message_text, 48)),
            preview_text: truncate_preview(&message_text, PREVIEW_MAX_CHARACTERS),
            payload_json: canonical_payload,
            raw_payload,
            test_run_identifier,
            received_at: None,
            attachments: Vec::new(),
        },
    )
}
