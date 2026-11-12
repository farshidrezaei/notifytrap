use serde::{Deserialize, Serialize};

use crate::domain::channel::ChannelType;
use crate::domain::errors::ApplicationError;
use crate::domain::verification_extractor::{VerificationArtifact, extract_verification_artifacts};

/// Processing state of a captured notification in the local inbox.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    Captured,
    Read,
    Archived,
}

impl NotificationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Captured => "captured",
            Self::Read => "read",
            Self::Archived => "archived",
        }
    }

    pub fn parse(raw_status: &str) -> Result<Self, ApplicationError> {
        match raw_status.trim().to_ascii_lowercase().as_str() {
            "captured" => Ok(Self::Captured),
            "read" => Ok(Self::Read),
            "archived" => Ok(Self::Archived),
            other => Err(ApplicationError::Validation(format!(
                "Unsupported notification status: {other}"
            ))),
        }
    }
}

/// Persisted notification attachment metadata returned without full binary payload bloat.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationAttachmentSummary {
    pub id: String,
    pub notification_id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub content_id: Option<String>,
    pub is_inline: bool,
    pub created_at: String,
}

/// Full attachment including binary content when exported or inspected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationAttachment {
    pub summary: NotificationAttachmentSummary,
    pub content_blob: Vec<u8>,
}

/// Serializable attachment payload returned over Tauri IPC for preview and disk download.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationAttachmentContent {
    pub summary: NotificationAttachmentSummary,
    pub content_base64: String,
}

/// HTTP header key/value entry returned from a strict loopback webhook replay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplayHeaderEntry {
    pub name: String,
    pub value: String,
}

/// Validated request to replay a captured webhook notification to a local loopback URL.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookReplayInput {
    pub notification_id: String,
    pub target_url: String,
}

/// Response snapshot captured from executing a strict loopback webhook replay.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookReplayResult {
    pub target_url: String,
    pub method: String,
    pub status_code: u16,
    pub duration_milliseconds: u64,
    pub response_headers: Vec<ReplayHeaderEntry>,
    pub response_body: String,
}

/// Normalized notification record captured for local inspection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub channel_id: String,
    pub channel_type: ChannelType,
    pub status: NotificationStatus,
    pub recipient: String,
    pub sender: String,
    pub subject: String,
    pub preview_text: String,
    pub payload_json: serde_json::Value,
    pub raw_payload: String,
    pub test_run_identifier: String,
    pub verification_artifacts: Vec<VerificationArtifact>,
    pub received_at: String,
    pub created_at: String,
}

/// Compact notification list entry used in paginated inbox views.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSummary {
    pub id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub channel_id: String,
    pub channel_type: ChannelType,
    pub status: NotificationStatus,
    pub recipient: String,
    pub sender: String,
    pub subject: String,
    pub preview_text: String,
    pub test_run_identifier: String,
    pub verification_artifacts: Vec<VerificationArtifact>,
    pub received_at: String,
}

/// Full detail view returned to the notification inspector.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDetail {
    pub notification: Notification,
    pub attachments: Vec<NotificationAttachmentSummary>,
}

impl Notification {
    pub fn to_summary(&self) -> NotificationSummary {
        NotificationSummary {
            id: self.id.clone(),
            workspace_id: self.workspace_id.clone(),
            project_id: self.project_id.clone(),
            environment_id: self.environment_id.clone(),
            channel_id: self.channel_id.clone(),
            channel_type: self.channel_type,
            status: self.status,
            recipient: self.recipient.clone(),
            sender: self.sender.clone(),
            subject: self.subject.clone(),
            preview_text: self.preview_text.clone(),
            test_run_identifier: self.test_run_identifier.clone(),
            verification_artifacts: self.verification_artifacts.clone(),
            received_at: self.received_at.clone(),
        }
    }

    pub fn populate_verification_artifacts(&mut self) {
        self.verification_artifacts = extract_verification_artifacts(
            self.channel_type,
            &self.subject,
            &self.preview_text,
            &self.payload_json,
            &self.raw_payload,
        );
    }
}

/// Input payload representing an attachment extracted during capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureAttachmentInput {
    pub filename: String,
    pub content_type: String,
    pub content_id: Option<String>,
    pub is_inline: bool,
    pub content_blob: Vec<u8>,
}

/// Input required to persist a captured notification atomically.
#[derive(Debug, Clone)]
pub struct CaptureNotificationInput {
    pub workspace_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub channel_id: String,
    pub channel_type: ChannelType,
    pub recipient: String,
    pub sender: String,
    pub subject: String,
    pub preview_text: String,
    pub payload_json: serde_json::Value,
    pub raw_payload: String,
    pub test_run_identifier: Option<String>,
    pub received_at: Option<String>,
    pub attachments: Vec<CaptureAttachmentInput>,
}

/// Strictly scoped inbox filter query.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationQuery {
    pub workspace_id: String,
    pub project_id: String,
    pub environment_id: String,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub channel_type: Option<ChannelType>,
    #[serde(default)]
    pub status: Option<NotificationStatus>,
    #[serde(default)]
    pub test_run_identifier: Option<String>,
    #[serde(default)]
    pub recipient_contains: Option<String>,
    #[serde(default)]
    pub subject_contains: Option<String>,
    #[serde(default)]
    pub since: Option<String>,
    #[serde(default)]
    pub search_query: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

/// Paginated inbox response.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPage {
    pub items: Vec<NotificationSummary>,
    pub total_count: u64,
    pub limit: u32,
    pub offset: u32,
}
