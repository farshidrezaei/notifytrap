use std::collections::BTreeMap;

use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::domain::errors::ApplicationError;

/// Supported local capture channel protocols.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Email,
    Sms,
    Push,
    Webhook,
}

impl ChannelType {
    pub const ALL: [ChannelType; 4] = [Self::Email, Self::Sms, Self::Push, Self::Webhook];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Sms => "sms",
            Self::Push => "push",
            Self::Webhook => "webhook",
        }
    }

    pub fn credential_prefix(self) -> &'static str {
        match self {
            Self::Email => "mail",
            Self::Sms => "sms",
            Self::Push => "push",
            Self::Webhook => "hook",
        }
    }

    pub fn parse(raw_type: &str) -> Result<Self, ApplicationError> {
        match raw_type.trim().to_ascii_lowercase().as_str() {
            "email" => Ok(Self::Email),
            "sms" => Ok(Self::Sms),
            "push" => Ok(Self::Push),
            "webhook" => Ok(Self::Webhook),
            other => Err(ApplicationError::Validation(format!(
                "Unsupported channel type: {other}"
            ))),
        }
    }
}

/// Custom HTTP mock response configuration applied to incoming webhook captures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookMockConfiguration {
    pub is_enabled: bool,
    pub status_code: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub body: String,
    pub delay_milliseconds: u64,
}

impl Default for WebhookMockConfiguration {
    fn default() -> Self {
        Self {
            is_enabled: false,
            status_code: 200,
            headers: BTreeMap::new(),
            body: r#"{"ok":true,"mockedBy":"notifytrap"}"#.to_string(),
            delay_milliseconds: 0,
        }
    }
}

/// Per-channel transient chaos / failure injection rule for testing retry & backoff resilience.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelChaosConfiguration {
    pub is_enabled: bool,
    pub latency_milliseconds: u64,
    pub failure_rate_percent: u8,
    pub http_error_status_code: u16,
    pub smtp_error_code: u16,
    pub error_message: String,
}

impl Default for ChannelChaosConfiguration {
    fn default() -> Self {
        Self {
            is_enabled: false,
            latency_milliseconds: 0,
            failure_rate_percent: 50,
            http_error_status_code: 503,
            smtp_error_code: 451,
            error_message: "Simulated transient delivery failure".to_string(),
        }
    }
}

/// Evaluates whether a probabilistic chaos failure triggers for the given percentage (0..=100).
pub fn should_trigger_chaos_failure(failure_rate_percent: u8) -> bool {
    match failure_rate_percent {
        0 => false,
        100..=u8::MAX => true,
        rate => rand::rng().random_range(0..100) < rate,
    }
}

/// Internal persisted channel row containing hashed and encrypted credential material.
/// This struct intentionally does not implement Serialize so secret fields never cross IPC by mistake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRecord {
    pub id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub name: String,
    pub channel_type: ChannelType,
    pub is_enabled: bool,
    pub credential_identifier: String,
    pub credential_hash: String,
    pub encrypted_credential: String,
    pub webhook_mock: WebhookMockConfiguration,
    pub chaos_rule: ChannelChaosConfiguration,
    pub created_at: String,
    pub updated_at: String,
}

impl ChannelRecord {
    pub fn to_summary(&self) -> ChannelSummary {
        ChannelSummary {
            id: self.id.clone(),
            workspace_id: self.workspace_id.clone(),
            project_id: self.project_id.clone(),
            environment_id: self.environment_id.clone(),
            name: self.name.clone(),
            channel_type: self.channel_type,
            is_enabled: self.is_enabled,
            credential_identifier: self.credential_identifier.clone(),
            webhook_mock: self.webhook_mock.clone(),
            chaos_rule: self.chaos_rule.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}

/// Safe channel representation returned in lists and overviews without secret values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSummary {
    pub id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub name: String,
    pub channel_type: ChannelType,
    pub is_enabled: bool,
    pub credential_identifier: String,
    #[serde(default)]
    pub webhook_mock: WebhookMockConfiguration,
    #[serde(default)]
    pub chaos_rule: ChannelChaosConfiguration,
    pub created_at: String,
    pub updated_at: String,
}

/// Decrypted credential payload returned only on explicit create, reveal, or rotate actions.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelCredentialReveal {
    pub channel_id: String,
    pub credential_identifier: String,
    pub secret_token: String,
    pub composite_key: String,
}

/// Combined channel metadata and initial credential returned on creation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreatedChannelResult {
    pub channel: ChannelSummary,
    pub credential: ChannelCredentialReveal,
}

/// Validated input required to create an isolated channel.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateChannelInput {
    pub workspace_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub name: String,
    pub channel_type: ChannelType,
    pub credential_identifier: Option<String>,
    pub is_enabled: Option<bool>,
}

/// Validated input for updating per-channel webhook mock and chaos simulation rules.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannelRulesInput {
    pub channel_id: String,
    pub webhook_mock: WebhookMockConfiguration,
    pub chaos_rule: ChannelChaosConfiguration,
}
