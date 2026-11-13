use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::channel::ChannelType;

/// Forwarding execution status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ForwardExecutionStatus {
    Success,
    Failed,
    Skipped,
}

/// Channel-level automatic forwarding rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelForwardingConfiguration {
    pub is_enabled: bool,
    pub target_url: String,
    #[serde(default)]
    pub custom_headers: BTreeMap<String, String>,
    pub include_original_headers: bool,
    pub filter_event_name: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for ChannelForwardingConfiguration {
    fn default() -> Self {
        Self {
            is_enabled: false,
            target_url: "http://127.0.0.1:8000/webhook".to_string(),
            custom_headers: BTreeMap::new(),
            include_original_headers: true,
            filter_event_name: None,
            timeout_seconds: 5,
        }
    }
}

/// Recorded outcome of executing an auto-forward dispatch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ForwardExecutionResult {
    pub notification_id: String,
    pub channel_id: String,
    pub channel_type: ChannelType,
    pub target_url: String,
    pub status: ForwardExecutionStatus,
    pub http_status_code: Option<u16>,
    pub duration_milliseconds: u64,
    pub response_body_snippet: Option<String>,
    pub error_message: Option<String>,
    pub executed_at: String,
}
