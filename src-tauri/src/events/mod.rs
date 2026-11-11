use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

use crate::domain::notification::NotificationSummary;

/// Canonical Tauri event names emitted from the Rust backend to the desktop UI.
pub struct EventChannels;

impl EventChannels {
    pub const NOTIFICATION_CAPTURED: &'static str = "notification:captured";
    pub const SERVER_STATUS_CHANGED: &'static str = "server:status_changed";
}

/// Event emitted immediately whenever a local HTTP or SMTP server stores a notification.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationCapturedEvent {
    pub notification: NotificationSummary,
    pub attachment_count: usize,
}

/// Event emitted when background capture server state or endpoints change.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatusChangedEvent {
    pub is_http_server_running: bool,
    pub is_smtp_server_running: bool,
    pub http_endpoint: String,
    pub smtp_endpoint: String,
}

/// Abstraction for broadcasting domain capture and lifecycle events to UI or test listeners.
pub trait EventPublisher: Send + Sync {
    fn emit_notification_captured(&self, payload: &NotificationCapturedEvent);
    fn emit_server_status_changed(&self, payload: &ServerStatusChangedEvent);
}

/// Tauri desktop IPC emitter forwarding backend events to active WebViews.
pub struct TauriEventPublisher {
    application_handle: AppHandle,
}

impl TauriEventPublisher {
    pub fn new(application_handle: AppHandle) -> Self {
        Self { application_handle }
    }
}

impl EventPublisher for TauriEventPublisher {
    fn emit_notification_captured(&self, payload: &NotificationCapturedEvent) {
        if let Err(error) = self
            .application_handle
            .emit(EventChannels::NOTIFICATION_CAPTURED, payload)
        {
            tracing::debug!("Unable to emit notification captured event: {error}");
        }
    }

    fn emit_server_status_changed(&self, payload: &ServerStatusChangedEvent) {
        if let Err(error) = self
            .application_handle
            .emit(EventChannels::SERVER_STATUS_CHANGED, payload)
        {
            tracing::debug!("Unable to emit server status changed event: {error}");
        }
    }
}

/// Deterministic in-memory event bus used in headless server and integration test suites.
pub struct BroadcastEventPublisher {
    notifications_sender: broadcast::Sender<NotificationCapturedEvent>,
    status_sender: broadcast::Sender<ServerStatusChangedEvent>,
}

impl Default for BroadcastEventPublisher {
    fn default() -> Self {
        Self::new(64)
    }
}

impl BroadcastEventPublisher {
    pub fn new(capacity: usize) -> Self {
        let (notifications_sender, _) = broadcast::channel(capacity);
        let (status_sender, _) = broadcast::channel(capacity);
        Self {
            notifications_sender,
            status_sender,
        }
    }

    pub fn subscribe_notifications(&self) -> broadcast::Receiver<NotificationCapturedEvent> {
        self.notifications_sender.subscribe()
    }

    pub fn subscribe_server_status(&self) -> broadcast::Receiver<ServerStatusChangedEvent> {
        self.status_sender.subscribe()
    }
}

impl EventPublisher for BroadcastEventPublisher {
    fn emit_notification_captured(&self, payload: &NotificationCapturedEvent) {
        let _ = self.notifications_sender.send(payload.clone());
    }

    fn emit_server_status_changed(&self, payload: &ServerStatusChangedEvent) {
        let _ = self.status_sender.send(payload.clone());
    }
}
