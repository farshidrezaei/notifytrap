use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::application::application_state::ApplicationServices;
use crate::domain::channel::{
    ChannelChaosConfiguration, ChannelSummary, ChannelType, WebhookMockConfiguration,
    should_trigger_chaos_failure,
};
use crate::domain::errors::ApplicationError;
use crate::domain::notification::CaptureNotificationInput;
use crate::domain::validation::{
    validate_bounded_text, validate_optional_test_run_identifier, validate_slug,
};
use crate::events::{EventPublisher, NotificationCapturedEvent};
use crate::infrastructure::http::mcp_routes::handle_mcp_json_rpc;
use crate::infrastructure::http::query_routes::{
    handle_get_notification_detail, handle_query_notifications, handle_wait_for_notification,
};

const HEADER_CHANNEL_KEY: &str = "x-channel-key";
const HEADER_AUTHORIZATION: &str = "authorization";
const HEADER_TEST_RUN_IDENTIFIER: &str = "x-notifytrap-run";
const REDACTED_SECRET_PLACEHOLDER: &str = "[REDACTED]";
const PREVIEW_MAX_CHARACTERS: usize = 160;

#[derive(Clone)]
pub struct HttpCaptureState {
    pub services: ApplicationServices,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub bound_endpoint: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureReceiptResponse {
    pub notification_id: String,
    pub channel_id: String,
    pub status: String,
    pub received_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SmsCapturePayload {
    pub to: String,
    pub message: String,
    pub from: Option<String>,
    pub metadata: Option<Value>,
}

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

#[derive(Debug, Deserialize)]
pub struct WebhookQueryParameters {
    pub channel_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct HttpErrorBody {
    code: String,
    message: String,
}

pub(super) struct HttpCaptureError {
    status_code: StatusCode,
    code: &'static str,
    message: String,
}

impl HttpCaptureError {
    pub(super) fn from_application_error(error: ApplicationError) -> Self {
        match error {
            ApplicationError::Unauthorized => Self {
                status_code: StatusCode::UNAUTHORIZED,
                code: "UNAUTHORIZED",
                message: "Channel authentication failed".to_string(),
            },
            ApplicationError::Validation(message) => Self {
                status_code: StatusCode::BAD_REQUEST,
                code: "VALIDATION_ERROR",
                message,
            },
            ApplicationError::NotFound(message) => Self {
                status_code: StatusCode::NOT_FOUND,
                code: "NOT_FOUND",
                message,
            },
            ApplicationError::Conflict(message) => Self {
                status_code: StatusCode::CONFLICT,
                code: "CONFLICT",
                message,
            },
            other => {
                tracing::error!("HTTP capture internal failure: {other}");
                Self {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR",
                    message: "Internal capture storage error".to_string(),
                }
            }
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            code: "VALIDATION_ERROR",
            message: message.into(),
        }
    }

    fn from_json_rejection(rejection: axum::extract::rejection::JsonRejection) -> Self {
        let status_code = rejection.status();
        let code = if status_code == StatusCode::PAYLOAD_TOO_LARGE {
            "PAYLOAD_TOO_LARGE"
        } else {
            "VALIDATION_ERROR"
        };
        Self {
            status_code,
            code,
            message: rejection.body_text(),
        }
    }

    pub(super) fn unauthorized() -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED",
            message: "Channel authentication failed".to_string(),
        }
    }

    pub(super) fn not_found(entity: &str) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            code: "NOT_FOUND",
            message: format!("{entity} not found"),
        }
    }

    pub(super) fn request_timeout(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::REQUEST_TIMEOUT,
            code: "WAIT_TIMEOUT",
            message: message.into(),
        }
    }

    pub(super) fn message(&self) -> &str {
        &self.message
    }

    pub(super) fn chaos_failure(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code,
            code: "CHAOS_SIMULATED_FAILURE",
            message,
        }
    }
}

impl IntoResponse for HttpCaptureError {
    fn into_response(self) -> Response {
        (
            self.status_code,
            Json(HttpErrorBody {
                code: self.code.to_string(),
                message: self.message,
            }),
        )
            .into_response()
    }
}

/// Builds the Axum router with payload size limits and isolated capture endpoints.
pub fn build_http_capture_router(state: HttpCaptureState, maximum_payload_bytes: usize) -> Router {
    Router::new()
        .route("/api/v1/health", get(handle_health))
        .route("/api/v1/mcp", post(handle_mcp_json_rpc))
        .route("/api/v1/notifications", get(handle_query_notifications))
        .route(
            "/api/v1/notifications/wait",
            get(handle_wait_for_notification),
        )
        .route(
            "/api/v1/notifications/{notification_id}",
            get(handle_get_notification_detail),
        )
        .route("/api/v1/sms", post(handle_sms_capture))
        .route("/api/v1/push", post(handle_push_capture))
        .route(
            "/api/v1/webhooks/{credential_identifier}",
            any(handle_webhook_root_capture),
        )
        .route(
            "/api/v1/webhooks/{credential_identifier}/{*subpath}",
            any(handle_webhook_subpath_capture),
        )
        .layer(DefaultBodyLimit::max(maximum_payload_bytes))
        .with_state(state)
}

async fn handle_health(State(state): State<HttpCaptureState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "notifytrap",
            "httpEndpoint": state.bound_endpoint,
        })),
    )
}

async fn handle_sms_capture(
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

async fn handle_push_capture(
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

async fn handle_webhook_root_capture(
    State(state): State<HttpCaptureState>,
    Path(credential_identifier): Path<String>,
    Query(query_parameters): Query<WebhookQueryParameters>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body_bytes: Bytes,
) -> Result<Response, HttpCaptureError> {
    process_webhook_capture(
        &state,
        &credential_identifier,
        None,
        &query_parameters,
        &method,
        &uri,
        &headers,
        &body_bytes,
    )
    .await
}

async fn handle_webhook_subpath_capture(
    State(state): State<HttpCaptureState>,
    Path((credential_identifier, subpath)): Path<(String, String)>,
    Query(query_parameters): Query<WebhookQueryParameters>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body_bytes: Bytes,
) -> Result<Response, HttpCaptureError> {
    process_webhook_capture(
        &state,
        &credential_identifier,
        Some(subpath.as_str()),
        &query_parameters,
        &method,
        &uri,
        &headers,
        &body_bytes,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn process_webhook_capture(
    state: &HttpCaptureState,
    credential_identifier: &str,
    subpath: Option<&str>,
    query_parameters: &WebhookQueryParameters,
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap,
    body_bytes: &Bytes,
) -> Result<Response, HttpCaptureError> {
    let validated_identifier = validate_slug(credential_identifier)
        .map_err(|error| HttpCaptureError::bad_request(error.to_string()))?;
    let channel =
        authenticate_webhook_channel(state, &validated_identifier, headers, query_parameters)?;
    apply_channel_chaos_if_enabled(&channel.chaos_rule).await?;
    let test_run_identifier = extract_test_run_identifier(headers)?;

    let raw_body_text = String::from_utf8_lossy(body_bytes).to_string();
    let parsed_body_json: Option<Value> = if raw_body_text.trim().is_empty() {
        None
    } else {
        serde_json::from_str(&raw_body_text).ok()
    };

    let redacted_headers = collect_redacted_headers(headers);
    let redacted_query = sanitize_query_string(uri.query());
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|header_value| header_value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let display_subpath = subpath
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("/{value}"))
        .unwrap_or_else(|| "/".to_string());

    let canonical_payload = json!({
        "method": method.as_str(),
        "path": uri.path(),
        "subpath": display_subpath,
        "query": redacted_query,
        "contentType": content_type,
        "headers": redacted_headers,
        "jsonBody": parsed_body_json,
        "textBody": raw_body_text,
        "bodyBytes": body_bytes.len(),
    });

    let preview_source = if raw_body_text.trim().is_empty() {
        format!("{} {}", method.as_str(), display_subpath)
    } else {
        raw_body_text.clone()
    };

    let webhook_mock = channel.webhook_mock.clone();
    let receipt = persist_and_emit(
        state,
        CaptureNotificationInput {
            workspace_id: channel.workspace_id,
            project_id: channel.project_id,
            environment_id: channel.environment_id,
            channel_id: channel.id,
            channel_type: ChannelType::Webhook,
            recipient: display_subpath.clone(),
            sender: method.as_str().to_string(),
            subject: format!("{} {}", method.as_str(), display_subpath),
            preview_text: truncate_preview(&preview_source, PREVIEW_MAX_CHARACTERS),
            payload_json: canonical_payload,
            raw_payload: raw_body_text,
            test_run_identifier,
            received_at: None,
            attachments: Vec::new(),
        },
    )?;

    if webhook_mock.is_enabled {
        return Ok(build_webhook_mock_response(&webhook_mock).await);
    }

    Ok(receipt.into_response())
}

async fn apply_channel_chaos_if_enabled(
    chaos_rule: &ChannelChaosConfiguration,
) -> Result<(), HttpCaptureError> {
    if !chaos_rule.is_enabled {
        return Ok(());
    }
    if chaos_rule.latency_milliseconds > 0 {
        tokio::time::sleep(Duration::from_millis(chaos_rule.latency_milliseconds)).await;
    }
    if should_trigger_chaos_failure(chaos_rule.failure_rate_percent) {
        let status = StatusCode::from_u16(chaos_rule.http_error_status_code)
            .unwrap_or(StatusCode::SERVICE_UNAVAILABLE);
        return Err(HttpCaptureError::chaos_failure(
            status,
            chaos_rule.error_message.clone(),
        ));
    }
    Ok(())
}

async fn build_webhook_mock_response(mock: &WebhookMockConfiguration) -> Response {
    if mock.delay_milliseconds > 0 {
        tokio::time::sleep(Duration::from_millis(mock.delay_milliseconds)).await;
    }
    let status = StatusCode::from_u16(mock.status_code).unwrap_or(StatusCode::OK);
    let mut builder = Response::builder().status(status);
    let mut has_content_type = false;

    for (name, value) in &mock.headers {
        if name.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            builder = builder.header(header_name, header_value);
        }
    }

    if !has_content_type && !mock.body.trim().is_empty() {
        builder = builder.header(axum::http::header::CONTENT_TYPE, "application/json");
    }

    builder
        .body(axum::body::Body::from(mock.body.clone()))
        .unwrap_or_else(|_| (status, mock.body.clone()).into_response())
}

fn persist_and_emit(
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

fn authenticate_header_channel(
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

fn authenticate_webhook_channel(
    state: &HttpCaptureState,
    path_identifier: &str,
    headers: &HeaderMap,
    query_parameters: &WebhookQueryParameters,
) -> Result<ChannelSummary, HttpCaptureError> {
    let presented_key = extract_channel_key_from_headers(headers)
        .or(query_parameters.channel_key.as_deref())
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

pub(super) fn extract_channel_key_from_headers(headers: &HeaderMap) -> Option<&str> {
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

fn extract_test_run_identifier(headers: &HeaderMap) -> Result<Option<String>, HttpCaptureError> {
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

fn collect_redacted_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
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

fn sanitize_query_string(raw_query: Option<&str>) -> BTreeMap<String, String> {
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

fn truncate_preview(text: &str, maximum_characters: usize) -> String {
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
