use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::channel::ChannelType;
use crate::domain::notification::CaptureNotificationInput;
use crate::domain::validation::{validate_bounded_text, validate_slug};
use crate::infrastructure::http::capture_auth::{
    PREVIEW_MAX_CHARACTERS, authenticate_header_channel, extract_test_run_identifier,
    persist_and_emit, truncate_preview,
};
use crate::infrastructure::http::capture_error::{CaptureReceiptResponse, HttpCaptureError};
use crate::infrastructure::http::routes::HttpCaptureState;
use crate::infrastructure::http::webhook_handler::apply_channel_chaos_if_enabled;

#[derive(Debug, Deserialize)]
pub struct PushCapturePayload {
    pub device_token: String,
    pub platform: String,
    pub title: String,
    pub body: String,
    pub data: Option<Value>,
    pub badge: Option<u64>,
    pub sound: Option<String>,
}

pub async fn handle_push_capture(
    State(state): State<HttpCaptureState>,
    headers: HeaderMap,
    payload_result: Result<Json<PushCapturePayload>, axum::extract::rejection::JsonRejection>,
) -> Result<(StatusCode, Json<CaptureReceiptResponse>), HttpCaptureError> {
    let channel = authenticate_header_channel(&headers, &state, ChannelType::Push)?;
    apply_channel_chaos_if_enabled(&channel.chaos_rule).await?;
    let test_run_identifier = extract_test_run_identifier(&headers)?;
    let Json(payload) = payload_result.map_err(HttpCaptureError::from_json_rejection)?;

    let device_token = validate_bounded_text(&payload.device_token, "device_token", 2, 512, false)
        .map_err(|error| HttpCaptureError::bad_request(error.to_string()))?;
    let platform = validate_slug(&payload.platform.trim().to_ascii_lowercase())
        .map_err(|error| HttpCaptureError::bad_request(error.to_string()))?;
    let title = validate_bounded_text(&payload.title, "title", 1, 256, false)
        .map_err(|error| HttpCaptureError::bad_request(error.to_string()))?;
    let body = validate_bounded_text(&payload.body, "body", 1, 16_000, true)
        .map_err(|error| HttpCaptureError::bad_request(error.to_string()))?;

    let canonical_payload = json!({
        "deviceToken": device_token,
        "platform": platform,
        "title": title,
        "body": body,
        "data": payload.data.unwrap_or_else(|| json!({})),
        "badge": payload.badge,
        "sound": payload.sound,
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
            channel_type: ChannelType::Push,
            recipient: device_token,
            sender: platform,
            subject: title,
            preview_text: truncate_preview(&body, PREVIEW_MAX_CHARACTERS),
            payload_json: canonical_payload,
            raw_payload,
            test_run_identifier,
            received_at: None,
            attachments: Vec::new(),
        },
    )
}
