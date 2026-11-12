use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    Link,
    Image,
}

/// Inspection outcome for an individual anchor link or remote image inside an email.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssetCheckResult {
    pub url: String,
    pub kind: AssetKind,
    pub status_code: Option<u16>,
    pub is_broken: bool,
    pub latency_milliseconds: u64,
    pub error_message: Option<String>,
}
