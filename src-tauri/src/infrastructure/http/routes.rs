use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{any, get, post};
use axum::{Json, Router};
use serde_json::json;

use crate::application::application_state::ApplicationServices;
use crate::events::EventPublisher;
use crate::infrastructure::http::mcp_routes::handle_mcp_json_rpc;
use crate::infrastructure::http::push_handler::handle_push_capture;
use crate::infrastructure::http::query_routes::{
    handle_get_notification_detail, handle_query_notifications, handle_wait_for_notification,
};
use crate::infrastructure::http::sms_handler::handle_sms_capture;
use crate::infrastructure::http::webhook_handler::{
    handle_webhook_root_capture, handle_webhook_subpath_capture,
};

pub use crate::infrastructure::http::capture_auth::extract_channel_key_from_headers;
pub use crate::infrastructure::http::capture_error::{CaptureReceiptResponse, HttpCaptureError};

#[derive(Clone)]
pub struct HttpCaptureState {
    pub services: ApplicationServices,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub bound_endpoint: String,
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
