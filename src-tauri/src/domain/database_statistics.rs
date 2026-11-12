use serde::{Deserialize, Serialize};

/// Comprehensive storage and disk footprint metrics for the SQLite database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStorageStatistics {
    pub database_path: Option<String>,
    pub file_size_bytes: u64,
    pub wal_size_bytes: u64,
    pub total_notifications_count: u64,
    pub total_attachments_count: u64,
    pub total_attachments_size_bytes: u64,
}
