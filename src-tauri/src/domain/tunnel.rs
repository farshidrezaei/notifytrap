use serde::{Deserialize, Serialize};

use crate::domain::errors::ApplicationError;

/// Supported local tunnel binary backends.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelProvider {
    Cloudflared,
    Ngrok,
    Bore,
    Custom,
}

impl TunnelProvider {
    pub const ALL: [TunnelProvider; 4] = [Self::Cloudflared, Self::Ngrok, Self::Bore, Self::Custom];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cloudflared => "cloudflared",
            Self::Ngrok => "ngrok",
            Self::Bore => "bore",
            Self::Custom => "custom",
        }
    }

    pub fn parse(raw_provider: &str) -> Result<Self, ApplicationError> {
        match raw_provider.trim().to_ascii_lowercase().as_str() {
            "cloudflared" => Ok(Self::Cloudflared),
            "ngrok" => Ok(Self::Ngrok),
            "bore" => Ok(Self::Bore),
            "custom" => Ok(Self::Custom),
            other => Err(ApplicationError::Validation(format!(
                "Unsupported tunnel provider: {other}"
            ))),
        }
    }
}

/// Operational state of the local tunnel child process.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Disconnected,
    Starting,
    Connected,
    Failed,
}

impl TunnelStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disconnected => "disconnected",
            Self::Starting => "starting",
            Self::Connected => "connected",
            Self::Failed => "failed",
        }
    }
}

/// Dynamic snapshot of current tunnel activity returned to Tauri IPC and UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatusReport {
    pub is_active: bool,
    pub status: TunnelStatus,
    pub provider: TunnelProvider,
    pub public_url: Option<String>,
    pub local_target: String,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
}

impl Default for TunnelStatusReport {
    fn default() -> Self {
        Self {
            is_active: false,
            status: TunnelStatus::Disconnected,
            provider: TunnelProvider::Cloudflared,
            public_url: None,
            local_target: "http://127.0.0.1:4080".to_string(),
            error_message: None,
            started_at: None,
        }
    }
}

/// Configuration payload when initiating or changing local ingress tunnel settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStartConfiguration {
    pub provider: TunnelProvider,
    pub custom_binary_path: Option<String>,
    pub authentication_token: Option<String>,
    pub custom_arguments: Option<String>,
}

impl Default for TunnelStartConfiguration {
    fn default() -> Self {
        Self {
            provider: TunnelProvider::Cloudflared,
            custom_binary_path: None,
            authentication_token: None,
            custom_arguments: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_driven_tunnel_provider_parsing() {
        let valid_test_cases = [
            ("cloudflared", TunnelProvider::Cloudflared),
            ("CLOUDFLARED", TunnelProvider::Cloudflared),
            ("  cloudflared  ", TunnelProvider::Cloudflared),
            ("ngrok", TunnelProvider::Ngrok),
            ("bore", TunnelProvider::Bore),
            ("custom", TunnelProvider::Custom),
        ];

        for (input_string, expected_provider) in valid_test_cases {
            let parsed = TunnelProvider::parse(input_string);
            assert!(parsed.is_ok(), "Expected valid parse for '{input_string}'");
            assert_eq!(parsed.unwrap(), expected_provider);
            assert_eq!(
                expected_provider.as_str(),
                input_string.trim().to_ascii_lowercase()
            );
        }
    }

    #[test]
    fn rejects_unsupported_tunnel_provider() {
        let invalid_providers = ["unknown", "localtunnel", "ssh", ""];
        for invalid_provider in invalid_providers {
            let parse_result = TunnelProvider::parse(invalid_provider);
            assert!(
                parse_result.is_err(),
                "Expected failure for '{invalid_provider}'"
            );
        }
    }

    #[test]
    fn tunnel_status_string_representation() {
        assert_eq!(TunnelStatus::Disconnected.as_str(), "disconnected");
        assert_eq!(TunnelStatus::Starting.as_str(), "starting");
        assert_eq!(TunnelStatus::Connected.as_str(), "connected");
        assert_eq!(TunnelStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn tunnel_status_report_serialization_round_trip() {
        let original_report = TunnelStatusReport {
            is_active: true,
            status: TunnelStatus::Connected,
            provider: TunnelProvider::Cloudflared,
            public_url: Some("https://notifytrap-test.trycloudflare.com".to_string()),
            local_target: "http://127.0.0.1:4080".to_string(),
            error_message: None,
            started_at: Some("2026-11-15T00:00:00Z".to_string()),
        };

        let serialized_json = serde_json::to_string(&original_report)
            .expect("serialization of tunnel status report should succeed");
        let deserialized_report: TunnelStatusReport = serde_json::from_str(&serialized_json)
            .expect("deserialization of tunnel status report should succeed");

        assert_eq!(original_report, deserialized_report);
    }
}
