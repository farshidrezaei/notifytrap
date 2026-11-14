use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::channel::{
    ChannelChaosConfiguration, ChannelType, WebhookMockConfiguration, should_trigger_chaos_failure,
};
use crate::domain::notification::CaptureNotificationInput;
use crate::domain::validation::validate_slug;
use crate::infrastructure::http::capture_auth::{
    PREVIEW_MAX_CHARACTERS, authenticate_webhook_channel, collect_redacted_headers,
    extract_test_run_identifier, persist_and_emit, sanitize_query_string, truncate_preview,
};
use crate::infrastructure::http::capture_error::HttpCaptureError;
use crate::infrastructure::http::forwarder::execute_channel_forwarding_if_enabled;
use crate::infrastructure::http::routes::HttpCaptureState;

#[derive(Debug, Deserialize)]
pub struct WebhookQueryParameters {
    pub channel_key: Option<String>,
}

pub async fn handle_webhook_root_capture(
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

pub async fn handle_webhook_subpath_capture(
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
pub async fn process_webhook_capture(
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
    let channel = authenticate_webhook_channel(
        state,
        &validated_identifier,
        headers,
        query_parameters.channel_key.as_deref(),
    )?;
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
            channel_id: channel.id.clone(),
            channel_type: ChannelType::Webhook,
            recipient: display_subpath.clone(),
            sender: method.as_str().to_string(),
            subject: format!("{} {}", method.as_str(), display_subpath),
            preview_text: truncate_preview(&preview_source, PREVIEW_MAX_CHARACTERS),
            payload_json: canonical_payload,
            raw_payload: raw_body_text.clone(),
            test_run_identifier,
            received_at: None,
            attachments: Vec::new(),
        },
    )?;

    if channel.forwarding_rule.is_enabled {
        let channel_id_clone = channel.id.clone();
        let notification_id = receipt.1.0.notification_id.clone();
        let database_manager = state.services.notifications.database_manager().clone();
        let publisher = Arc::clone(&state.event_publisher);
        let raw_bytes_vec = body_bytes.to_vec();

        tokio::spawn(async move {
            let record = database_manager.with_connection(|connection| {
                crate::infrastructure::database::channel_repository::SqliteChannelRepository::find_by_id(
                    connection,
                    &channel_id_clone,
                )
            });
            let notification = database_manager.with_connection(|connection| {
                crate::infrastructure::database::notification_repository::SqliteNotificationRepository::find_by_id(
                    connection,
                    &notification_id,
                )
            });

            if let (Ok(Some(ch)), Ok(Some(notif))) = (record, notification)
                && let Some(forward_result) =
                    execute_channel_forwarding_if_enabled(&ch, &notif, &raw_bytes_vec).await
            {
                publisher.emit_forward_executed(&forward_result);
            }
        });
    }

    if webhook_mock.is_enabled {
        return Ok(build_webhook_mock_response(&webhook_mock).await);
    }

    Ok(receipt.into_response())
}

pub async fn apply_channel_chaos_if_enabled(
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

pub async fn build_webhook_mock_response(mock: &WebhookMockConfiguration) -> Response {
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
