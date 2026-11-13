use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::domain::channel::ChannelRecord;
use crate::domain::forwarding::{
    ChannelForwardingConfiguration, ForwardExecutionResult, ForwardExecutionStatus,
};
use crate::domain::notification::Notification;
use crate::domain::validation::current_utc_timestamp;

const MAXIMUM_SNIPPET_BYTES: usize = 2048;

/// Executes an asynchronous auto-forwarding HTTP request for a captured notification if configured on the channel.
pub async fn execute_channel_forwarding_if_enabled(
    channel: &ChannelRecord,
    notification: &Notification,
    raw_payload_bytes: &[u8],
) -> Option<ForwardExecutionResult> {
    let rule = &channel.forwarding_rule;
    if !rule.is_enabled {
        return None;
    }

    // Evaluate optional event filter for webhooks or json payloads
    if let Some(expected_event) = &rule.filter_event_name {
        let expected = expected_event.trim();
        if !expected.is_empty() {
            let matched = matches_event_filter(notification, expected);
            if !matched {
                return Some(ForwardExecutionResult {
                    notification_id: notification.id.clone(),
                    channel_id: channel.id.clone(),
                    channel_type: channel.channel_type,
                    target_url: rule.target_url.clone(),
                    status: ForwardExecutionStatus::Skipped,
                    http_status_code: None,
                    duration_milliseconds: 0,
                    response_body_snippet: Some(format!(
                        "Skipped: event filter '{expected}' did not match notification"
                    )),
                    error_message: None,
                    executed_at: current_utc_timestamp(),
                });
            }
        }
    }

    let result = dispatch_forward_request(rule, notification, raw_payload_bytes).await;
    Some(result)
}

fn matches_event_filter(notification: &Notification, expected_event: &str) -> bool {
    let payload = &notification.payload_json;
    if let Some(event_val) = payload.get("event").and_then(|value| value.as_str())
        && event_val.eq_ignore_ascii_case(expected_event)
    {
        return true;
    }
    if let Some(type_val) = payload.get("type").and_then(|value| value.as_str())
        && type_val.eq_ignore_ascii_case(expected_event)
    {
        return true;
    }
    notification.subject.eq_ignore_ascii_case(expected_event)
}

async fn dispatch_forward_request(
    rule: &ChannelForwardingConfiguration,
    notification: &Notification,
    raw_payload_bytes: &[u8],
) -> ForwardExecutionResult {
    let started_instant = Instant::now();
    let timeout_duration = Duration::from_secs(rule.timeout_seconds.max(1));
    let client_builder = reqwest::Client::builder().timeout(timeout_duration);

    let client = match client_builder.build() {
        Ok(client) => client,
        Err(build_error) => {
            return ForwardExecutionResult {
                notification_id: notification.id.clone(),
                channel_id: notification.channel_id.clone(),
                channel_type: notification.channel_type,
                target_url: rule.target_url.clone(),
                status: ForwardExecutionStatus::Failed,
                http_status_code: None,
                duration_milliseconds: started_instant.elapsed().as_millis() as u64,
                response_body_snippet: None,
                error_message: Some(format!(
                    "Failed to build HTTP forwarding client: {build_error}"
                )),
                executed_at: current_utc_timestamp(),
            };
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-forwarded-by"),
        HeaderValue::from_static("notifytrap"),
    );
    headers.insert(
        HeaderName::from_static("x-notifytrap-notification-id"),
        HeaderValue::from_str(&notification.id).unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    headers.insert(
        HeaderName::from_static("x-notifytrap-channel-type"),
        HeaderValue::from_static(notification.channel_type.as_str()),
    );

    if !notification.test_run_identifier.is_empty()
        && let Ok(value) = HeaderValue::from_str(&notification.test_run_identifier)
    {
        headers.insert(HeaderName::from_static("x-notifytrap-run"), value);
    }

    for (name, val) in &rule.custom_headers {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(val),
        ) {
            headers.insert(header_name, header_value);
        }
    }

    let payload_to_send = if !raw_payload_bytes.is_empty() {
        raw_payload_bytes.to_vec()
    } else if !notification.payload_json.is_null() {
        if !headers.contains_key("content-type") {
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }
        notification.payload_json.to_string().into_bytes()
    } else {
        notification.preview_text.as_bytes().to_vec()
    };

    let request = client
        .post(&rule.target_url)
        .headers(headers)
        .body(payload_to_send);

    match request.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let duration_milliseconds = started_instant.elapsed().as_millis() as u64;
            let response_bytes = response.bytes().await.unwrap_or_default();
            let snippet = if response_bytes.is_empty() {
                None
            } else {
                let text = String::from_utf8_lossy(&response_bytes);
                Some(text.chars().take(MAXIMUM_SNIPPET_BYTES).collect())
            };

            let execution_status = if (200..=299).contains(&status_code) {
                ForwardExecutionStatus::Success
            } else {
                ForwardExecutionStatus::Failed
            };

            ForwardExecutionResult {
                notification_id: notification.id.clone(),
                channel_id: notification.channel_id.clone(),
                channel_type: notification.channel_type,
                target_url: rule.target_url.clone(),
                status: execution_status,
                http_status_code: Some(status_code),
                duration_milliseconds,
                response_body_snippet: snippet,
                error_message: if execution_status == ForwardExecutionStatus::Failed {
                    Some(format!("Forward destination returned HTTP {status_code}"))
                } else {
                    None
                },
                executed_at: current_utc_timestamp(),
            }
        }
        Err(request_error) => {
            let duration_milliseconds = started_instant.elapsed().as_millis() as u64;
            ForwardExecutionResult {
                notification_id: notification.id.clone(),
                channel_id: notification.channel_id.clone(),
                channel_type: notification.channel_type,
                target_url: rule.target_url.clone(),
                status: ForwardExecutionStatus::Failed,
                http_status_code: None,
                duration_milliseconds,
                response_body_snippet: None,
                error_message: Some(format!("Forward request failed: {request_error}")),
                executed_at: current_utc_timestamp(),
            }
        }
    }
}
