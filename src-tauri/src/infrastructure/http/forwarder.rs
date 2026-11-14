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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel::{ChannelChaosConfiguration, WebhookMockConfiguration};
    use crate::domain::notification::NotificationStatus;
    use serde_json::json;

    fn create_test_notification(payload: serde_json::Value, subject: &str) -> Notification {
        Notification {
            id: "notification-test-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            project_id: "project-1".to_string(),
            environment_id: "environment-1".to_string(),
            channel_id: "channel-1".to_string(),
            channel_type: crate::domain::channel::ChannelType::Webhook,
            status: NotificationStatus::Captured,
            recipient: "target@example.com".to_string(),
            sender: "sender@example.com".to_string(),
            subject: subject.to_string(),
            preview_text: "Preview".to_string(),
            payload_json: payload,
            raw_payload: "{}".to_string(),
            test_run_identifier: "run-42".to_string(),
            verification_artifacts: Vec::new(),
            received_at: "2026-11-15T00:00:00Z".to_string(),
            created_at: "2026-11-15T00:00:00Z".to_string(),
        }
    }

    fn create_test_channel_record(
        forwarding_rule: ChannelForwardingConfiguration,
    ) -> ChannelRecord {
        ChannelRecord {
            id: "channel-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            project_id: "project-1".to_string(),
            environment_id: "environment-1".to_string(),
            name: "Forwarding Test Channel".to_string(),
            channel_type: crate::domain::channel::ChannelType::Webhook,
            is_enabled: true,
            credential_identifier: "webhook-test".to_string(),
            credential_hash: "hash".to_string(),
            encrypted_credential: "enc".to_string(),
            webhook_mock: WebhookMockConfiguration::default(),
            chaos_rule: ChannelChaosConfiguration::default(),
            forwarding_rule,
            created_at: "2026-11-15T00:00:00Z".to_string(),
            updated_at: "2026-11-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn matches_event_filter_evaluates_event_key_case_insensitively() {
        let notification = create_test_notification(json!({"event": "payment.succeeded"}), "");
        assert!(matches_event_filter(&notification, "payment.succeeded"));
        assert!(matches_event_filter(&notification, "PAYMENT.SUCCEEDED"));
        assert!(!matches_event_filter(&notification, "user.created"));
    }

    #[test]
    fn matches_event_filter_evaluates_type_key() {
        let notification = create_test_notification(json!({"type": "invoice.paid"}), "");
        assert!(matches_event_filter(&notification, "invoice.paid"));
        assert!(!matches_event_filter(&notification, "invoice.failed"));
    }

    #[test]
    fn matches_event_filter_evaluates_subject_fallback() {
        let notification = create_test_notification(json!({"message": "Hello"}), "Password Reset");
        assert!(matches_event_filter(&notification, "Password Reset"));
        assert!(matches_event_filter(&notification, "password reset"));
        assert!(!matches_event_filter(&notification, "Welcome"));
    }

    #[tokio::test]
    async fn execute_channel_forwarding_returns_none_when_disabled() {
        let rule = ChannelForwardingConfiguration {
            is_enabled: false,
            ..Default::default()
        };
        let channel = create_test_channel_record(rule);
        let notification = create_test_notification(json!({}), "Test");

        let result = execute_channel_forwarding_if_enabled(&channel, &notification, b"").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn execute_channel_forwarding_returns_skipped_when_filter_mismatch() {
        let rule = ChannelForwardingConfiguration {
            is_enabled: true,
            filter_event_name: Some("order.completed".to_string()),
            ..Default::default()
        };
        let channel = create_test_channel_record(rule);
        let notification = create_test_notification(json!({"event": "order.cancelled"}), "Test");

        let result = execute_channel_forwarding_if_enabled(&channel, &notification, b"").await;
        assert!(result.is_some());
        let execution = result.unwrap();
        assert_eq!(execution.status, ForwardExecutionStatus::Skipped);
        assert!(
            execution
                .response_body_snippet
                .unwrap()
                .contains("Skipped: event filter")
        );
    }
}
