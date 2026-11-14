use std::collections::BTreeMap;

use axum::Json;
use axum::http::{HeaderMap, StatusCode};

use crate::domain::channel::{ChannelSummary, ChannelType};
use crate::domain::notification::CaptureNotificationInput;
use crate::domain::validation::validate_optional_test_run_identifier;
use crate::events::NotificationCapturedEvent;
use crate::infrastructure::http::capture_error::{CaptureReceiptResponse, HttpCaptureError};
use crate::infrastructure::http::routes::HttpCaptureState;

pub const HEADER_CHANNEL_KEY: &str = "x-channel-key";
pub const HEADER_AUTHORIZATION: &str = "authorization";
pub const HEADER_TEST_RUN_IDENTIFIER: &str = "x-notifytrap-run";
pub const REDACTED_SECRET_PLACEHOLDER: &str = "[REDACTED]";
pub const PREVIEW_MAX_CHARACTERS: usize = 160;

pub fn persist_and_emit(
    state: &HttpCaptureState,
    capture_input: CaptureNotificationInput,
) -> Result<(StatusCode, Json<CaptureReceiptResponse>), HttpCaptureError> {
    let detail = state
        .services
        .notifications
        .capture_notification(capture_input)
        .map_err(HttpCaptureError::from_application_error)?;

    state
        .event_publisher
        .emit_notification_captured(&NotificationCapturedEvent {
            notification: detail.notification.to_summary(),
            attachment_count: detail.attachments.len(),
        });

    Ok((
        StatusCode::ACCEPTED,
        Json(CaptureReceiptResponse {
            notification_id: detail.notification.id,
            channel_id: detail.notification.channel_id,
            status: detail.notification.status.as_str().to_string(),
            received_at: detail.notification.received_at,
        }),
    ))
}

pub fn authenticate_header_channel(
    headers: &HeaderMap,
    state: &HttpCaptureState,
    expected_type: ChannelType,
) -> Result<ChannelSummary, HttpCaptureError> {
    let composite_key =
        extract_channel_key_from_headers(headers).ok_or_else(HttpCaptureError::unauthorized)?;

    let channel = state
        .services
        .channels
        .authenticate_composite_key(composite_key)
        .map_err(HttpCaptureError::from_application_error)?;

    if channel.channel_type != expected_type {
        return Err(HttpCaptureError::unauthorized());
    }

    Ok(channel)
}

pub fn authenticate_webhook_channel(
    state: &HttpCaptureState,
    path_identifier: &str,
    headers: &HeaderMap,
    query_channel_key: Option<&str>,
) -> Result<ChannelSummary, HttpCaptureError> {
    let presented_key = extract_channel_key_from_headers(headers)
        .or(query_channel_key)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(HttpCaptureError::unauthorized)?;

    let channel = if let Some((key_identifier, secret)) = presented_key.split_once('.') {
        if key_identifier != path_identifier {
            return Err(HttpCaptureError::unauthorized());
        }
        state
            .services
            .channels
            .authenticate_identifier_and_secret(path_identifier, secret)
            .map_err(HttpCaptureError::from_application_error)?
    } else {
        state
            .services
            .channels
            .authenticate_identifier_and_secret(path_identifier, presented_key)
            .map_err(HttpCaptureError::from_application_error)?
    };

    if channel.channel_type != ChannelType::Webhook {
        return Err(HttpCaptureError::unauthorized());
    }

    Ok(channel)
}

pub fn extract_channel_key_from_headers(headers: &HeaderMap) -> Option<&str> {
    if let Some(channel_key_header) = headers
        .get(HEADER_CHANNEL_KEY)
        .and_then(|value| value.to_str().ok())
    {
        let trimmed = channel_key_header.trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    if let Some(authorization_header) = headers
        .get(HEADER_AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        let trimmed = authorization_header.trim();
        if let Some(bearer_token) = trimmed
            .strip_prefix("Bearer ")
            .or_else(|| trimmed.strip_prefix("bearer "))
        {
            let token = bearer_token.trim();
            if !token.is_empty() {
                return Some(token);
            }
        }
    }

    None
}

pub fn extract_test_run_identifier(
    headers: &HeaderMap,
) -> Result<Option<String>, HttpCaptureError> {
    let raw_header = headers
        .get(HEADER_TEST_RUN_IDENTIFIER)
        .and_then(|value| value.to_str().ok());
    validate_optional_test_run_identifier(raw_header)
        .map_err(|error| HttpCaptureError::bad_request(error.to_string()))
}

fn is_sensitive_header_name(lowercase_name: &str) -> bool {
    matches!(
        lowercase_name,
        HEADER_CHANNEL_KEY
            | HEADER_AUTHORIZATION
            | "proxy-authorization"
            | "x-api-key"
            | "cookie"
            | "set-cookie"
    )
}

fn is_sensitive_query_key(lowercase_key: &str) -> bool {
    matches!(
        lowercase_key,
        "channel_key" | "token" | "api_key" | "apikey" | "secret" | "password" | "authorization"
    )
}

pub fn collect_redacted_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (header_name, header_value) in headers {
        let lowercase_name = header_name.as_str().to_ascii_lowercase();
        let value = if is_sensitive_header_name(&lowercase_name) {
            REDACTED_SECRET_PLACEHOLDER.to_string()
        } else {
            header_value
                .to_str()
                .map(str::to_string)
                .unwrap_or_else(|_| "<binary>".to_string())
        };
        map.insert(lowercase_name, value);
    }
    map
}

pub fn sanitize_query_string(raw_query: Option<&str>) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let Some(query_text) = raw_query else {
        return map;
    };

    for pair in query_text.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let lowercase_key = key.to_ascii_lowercase();
        let sanitized_value = if is_sensitive_query_key(&lowercase_key) {
            REDACTED_SECRET_PLACEHOLDER.to_string()
        } else {
            value.to_string()
        };
        map.insert(key.to_string(), sanitized_value);
    }
    map
}

pub fn truncate_preview(text: &str, maximum_characters: usize) -> String {
    let compact: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result = String::new();
    for (index, character) in compact.chars().enumerate() {
        if index >= maximum_characters {
            result.push('…');
            break;
        }
        result.push(character);
    }
    result
}
