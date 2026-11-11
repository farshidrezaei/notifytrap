use serde::{Deserialize, Serialize};

use crate::configuration::server_configuration::ServerConfiguration;

/// Top-level immutable-by-default runtime configuration for NotifyTrap.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationConfiguration {
    pub application_name: String,
    pub application_version: String,
    pub default_page_size: usize,
    pub maximum_page_size: usize,
    pub server: ServerConfiguration,
}

impl ApplicationConfiguration {
    pub const APPLICATION_NAME: &'static str = "NotifyTrap";
    pub const DATABASE_FILENAME: &'static str = "notifytrap.sqlite3";
    pub const DEFAULT_PAGE_SIZE: usize = 50;
    pub const MAXIMUM_PAGE_SIZE: usize = 200;
}

impl Default for ApplicationConfiguration {
    fn default() -> Self {
        Self {
            application_name: Self::APPLICATION_NAME.to_string(),
            application_version: env!("CARGO_PKG_VERSION").to_string(),
            default_page_size: Self::DEFAULT_PAGE_SIZE,
            maximum_page_size: Self::MAXIMUM_PAGE_SIZE,
            server: ServerConfiguration::default(),
        }
    }
}
