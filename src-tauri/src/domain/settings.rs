use serde::{Deserialize, Serialize};

/// Persisted key-value application setting entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSettingEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: String,
}
