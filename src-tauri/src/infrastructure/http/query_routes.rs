use std::time::Duration;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

use crate::domain::channel::ChannelSummary;
use crate::domain::notification::{
    NotificationDetail, NotificationPage, NotificationQuery, NotificationStatus,
};
use crate::infrastructure::http::routes::{
    HttpCaptureError, HttpCaptureState, extract_channel_key_from_headers,
};

const DEFAULT_WAIT_TIMEOUT_MILLISECONDS: u64 = 5_000;
const MINIMUM_WAIT_TIMEOUT_MILLISECONDS: u64 = 50;
const MAXIMUM_WAIT_TIMEOUT_MILLISECONDS: u64 = 30_000;

/// Query string filters supported by local test-automation HTTP endpoints.
#[derive(Debug, Deserialize, Default)]
pub struct HttpNotificationQueryParameters {
    pub status: Option<String>,
    pub run_id: Option<String>,
    pub recipient: Option<String>,
    pub subject: Option<String>,
    pub since: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub timeout_ms: Option<u64>,
}

/// Lists notifications scoped strictly to the authenticated channel key.
pub(super) async fn handle_query_notifications(
    State(state): State<HttpCaptureState>,
    headers: HeaderMap,
    Query(parameters): Query<HttpNotificationQueryParameters>,
) -> Result<(StatusCode, Json<NotificationPage>), HttpCaptureError> {
    let channel = authenticate_any_channel(&headers, &state)?;
    let query = build_channel_query(&channel, &parameters)?;
    let page = state
        .services
        .notifications
        .query_notifications(query)
        .map_err(HttpCaptureError::from_application_error)?;
    Ok((StatusCode::OK, Json(page)))
}

/// Returns full detail for one notification owned by the authenticated channel.
pub(super) async fn handle_get_notification_detail(
    State(state): State<HttpCaptureState>,
    headers: HeaderMap,
    Path(notification_id): Path<String>,
) -> Result<(StatusCode, Json<NotificationDetail>), HttpCaptureError> {
    let channel = authenticate_any_channel(&headers, &state)?;
    let detail = state
        .services
        .notifications
        .get_notification_detail(&notification_id)
        .map_err(HttpCaptureError::from_application_error)?;

    if detail.notification.channel_id != channel.id {
        return Err(HttpCaptureError::not_found("Notification"));
    }

    Ok((StatusCode::OK, Json(detail)))
}

/// Long-polls without busy-sleeping until a matching notification arrives or timeout elapses.
pub(super) async fn handle_wait_for_notification(
    State(state): State<HttpCaptureState>,
    headers: HeaderMap,
    Query(parameters): Query<HttpNotificationQueryParameters>,
) -> Result<(StatusCode, Json<NotificationDetail>), HttpCaptureError> {
    let channel = authenticate_any_channel(&headers, &state)?;
    let mut query = build_channel_query(&channel, &parameters)?;
    query.limit = Some(1);
    query.offset = Some(0);

    let timeout_milliseconds = parameters
        .timeout_ms
        .unwrap_or(DEFAULT_WAIT_TIMEOUT_MILLISECONDS)
        .clamp(
            MINIMUM_WAIT_TIMEOUT_MILLISECONDS,
            MAXIMUM_WAIT_TIMEOUT_MILLISECONDS,
        );

    // Subscribe before the initial database lookup so captures arriving concurrently are never missed.
    let mut receiver = state.event_publisher.subscribe_notifications();

    if let Some(detail) = try_find_latest_matching_detail(&state, &query)? {
        return Ok((StatusCode::OK, Json(detail)));
    }

    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_milliseconds);

    loop {
        tokio::select! {
            _ = tokio::time::sleep_until(deadline) => {
                return Err(HttpCaptureError::request_timeout(
                    "No matching notification captured before timeout",
                ));
            }
            receive_outcome = receiver.recv() => {
                match receive_outcome {
                    Ok(event) if event.notification.channel_id == channel.id => {
                        if let Some(detail) = try_find_latest_matching_detail(&state, &query)? {
                            return Ok((StatusCode::OK, Json(detail)));
                        }
                    }
                    Ok(_) => {}
                    Err(RecvError::Lagged(_)) => {
                        if let Some(detail) = try_find_latest_matching_detail(&state, &query)? {
                            return Ok((StatusCode::OK, Json(detail)));
                        }
                    }
                    Err(RecvError::Closed) => {
                        return Err(HttpCaptureError::request_timeout(
                            "Notification event stream closed before match",
                        ));
                    }
                }
            }
        }
    }
}

fn try_find_latest_matching_detail(
    state: &HttpCaptureState,
    query: &NotificationQuery,
) -> Result<Option<NotificationDetail>, HttpCaptureError> {
    let page = state
        .services
        .notifications
        .query_notifications(query.clone())
        .map_err(HttpCaptureError::from_application_error)?;

    let Some(summary) = page.items.first() else {
        return Ok(None);
    };

    let detail = state
        .services
        .notifications
        .get_notification_detail(&summary.id)
        .map_err(HttpCaptureError::from_application_error)?;
    Ok(Some(detail))
}

fn authenticate_any_channel(
    headers: &HeaderMap,
    state: &HttpCaptureState,
) -> Result<ChannelSummary, HttpCaptureError> {
    let composite_key =
        extract_channel_key_from_headers(headers).ok_or_else(HttpCaptureError::unauthorized)?;

    state
        .services
        .channels
        .authenticate_composite_key(composite_key)
        .map_err(HttpCaptureError::from_application_error)
}

fn build_channel_query(
    channel: &ChannelSummary,
    parameters: &HttpNotificationQueryParameters,
) -> Result<NotificationQuery, HttpCaptureError> {
    let status = match parameters
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(raw_status) => Some(
            NotificationStatus::parse(raw_status)
                .map_err(HttpCaptureError::from_application_error)?,
        ),
        None => None,
    };

    Ok(NotificationQuery {
        workspace_id: channel.workspace_id.clone(),
        project_id: channel.project_id.clone(),
        environment_id: channel.environment_id.clone(),
        channel_id: Some(channel.id.clone()),
        channel_type: Some(channel.channel_type),
        status,
        test_run_identifier: parameters.run_id.clone(),
        recipient_contains: parameters.recipient.clone(),
        subject_contains: parameters.subject.clone(),
        since: parameters.since.clone(),
        search_query: parameters.search.clone(),
        limit: parameters.limit,
        offset: parameters.offset,
    })
}
