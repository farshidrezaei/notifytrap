use serde::{Deserialize, Serialize};

/// Configuration for local background capture servers (loopback-bound by default).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfiguration {
    pub bind_address: String,
    pub http_port: u16,
    pub smtp_port: u16,
    pub maximum_http_payload_bytes: usize,
    pub maximum_smtp_message_bytes: usize,
    pub shutdown_grace_period_seconds: u64,
}

impl ServerConfiguration {
    pub const DEFAULT_BIND_ADDRESS: &'static str = "127.0.0.1";
    pub const DEFAULT_HTTP_PORT: u16 = 4080;
    pub const DEFAULT_SMTP_PORT: u16 = 1025;
    pub const DEFAULT_MAXIMUM_HTTP_PAYLOAD_BYTES: usize = 2 * 1024 * 1024;
    pub const DEFAULT_MAXIMUM_SMTP_MESSAGE_BYTES: usize = 10 * 1024 * 1024;
    pub const DEFAULT_SHUTDOWN_GRACE_PERIOD_SECONDS: u64 = 5;
}

impl Default for ServerConfiguration {
    fn default() -> Self {
        Self {
            bind_address: Self::DEFAULT_BIND_ADDRESS.to_string(),
            http_port: Self::DEFAULT_HTTP_PORT,
            smtp_port: Self::DEFAULT_SMTP_PORT,
            maximum_http_payload_bytes: Self::DEFAULT_MAXIMUM_HTTP_PAYLOAD_BYTES,
            maximum_smtp_message_bytes: Self::DEFAULT_MAXIMUM_SMTP_MESSAGE_BYTES,
            shutdown_grace_period_seconds: Self::DEFAULT_SHUTDOWN_GRACE_PERIOD_SECONDS,
        }
    }
}
