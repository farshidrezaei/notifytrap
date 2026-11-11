use crate::configuration::server_configuration::ServerConfiguration;
use crate::domain::errors::ApplicationError;
use crate::domain::settings::ApplicationSettingEntry;
use crate::domain::validation::{
    MAXIMUM_PAYLOAD_BYTES, MINIMUM_PAYLOAD_BYTES, current_utc_timestamp, validate_bind_address,
    validate_grace_period_seconds, validate_retention_days, validate_tcp_port, validate_uuid,
};
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::database::settings_repository::SqliteSettingsRepository;

pub const SERVER_CONFIGURATION_KEY: &str = "server_configuration";
pub const RETENTION_DAYS_SETTING_KEY: &str = "inbox.retention_days";
pub const AUTO_UPDATE_CHECK_SETTING_KEY: &str = "app.auto_update_check_enabled";
const WORKSPACE_SELECTION_SETTING_KEY: &str = "workspace.active_selection";
const DEFAULT_RETENTION_DAYS: u64 = 30;

/// Orchestrates validated local application settings persistence.
#[derive(Clone)]
pub struct SettingsService {
    database_manager: DatabaseConnectionManager,
}

impl SettingsService {
    pub fn new(database_manager: DatabaseConnectionManager) -> Self {
        Self { database_manager }
    }

    pub fn load_server_configuration_or_default(
        &self,
        fallback: &ServerConfiguration,
    ) -> Result<ServerConfiguration, ApplicationError> {
        let maybe_entry = self.database_manager.with_connection(|connection| {
            SqliteSettingsRepository::find_by_key(connection, SERVER_CONFIGURATION_KEY)
        })?;

        match maybe_entry {
            Some(entry) => serde_json::from_value(entry.value).map_err(|error| {
                ApplicationError::Internal(format!(
                    "Corrupted server configuration setting: {error}"
                ))
            }),
            None => Ok(fallback.clone()),
        }
    }

    pub fn save_server_configuration(
        &self,
        mut configuration: ServerConfiguration,
    ) -> Result<ServerConfiguration, ApplicationError> {
        configuration.bind_address = validate_bind_address(&configuration.bind_address)?;
        configuration.http_port = validate_tcp_port(configuration.http_port, "HTTP")?;
        configuration.smtp_port = validate_tcp_port(configuration.smtp_port, "SMTP")?;

        if configuration.http_port == configuration.smtp_port {
            return Err(ApplicationError::Validation(
                "HTTP port and SMTP port must be different".to_string(),
            ));
        }

        if !(MINIMUM_PAYLOAD_BYTES..=MAXIMUM_PAYLOAD_BYTES)
            .contains(&configuration.maximum_http_payload_bytes)
        {
            return Err(ApplicationError::Validation(
                "Maximum HTTP payload size is out of allowed bounds".to_string(),
            ));
        }

        if !(MINIMUM_PAYLOAD_BYTES..=MAXIMUM_PAYLOAD_BYTES)
            .contains(&configuration.maximum_smtp_message_bytes)
        {
            return Err(ApplicationError::Validation(
                "Maximum SMTP message size is out of allowed bounds".to_string(),
            ));
        }

        configuration.shutdown_grace_period_seconds =
            validate_grace_period_seconds(configuration.shutdown_grace_period_seconds)?;

        let value = serde_json::to_value(&configuration).map_err(|error| {
            ApplicationError::Internal(format!("Failed to serialize server configuration: {error}"))
        })?;
        self.set_setting(SERVER_CONFIGURATION_KEY, value)?;
        Ok(configuration)
    }

    pub fn get_setting(
        &self,
        raw_key: &str,
    ) -> Result<Option<ApplicationSettingEntry>, ApplicationError> {
        let key = validate_setting_key(raw_key)?;
        self.database_manager
            .with_connection(|connection| SqliteSettingsRepository::find_by_key(connection, &key))
    }

    pub fn set_setting(
        &self,
        raw_key: &str,
        value: serde_json::Value,
    ) -> Result<ApplicationSettingEntry, ApplicationError> {
        let key = validate_setting_key(raw_key)?;
        validate_known_setting_value(&key, &value)?;
        let entry = ApplicationSettingEntry {
            key,
            value,
            updated_at: current_utc_timestamp(),
        };
        self.database_manager
            .with_connection(|connection| SqliteSettingsRepository::upsert(connection, &entry))?;
        Ok(entry)
    }

    pub fn list_all_settings(&self) -> Result<Vec<ApplicationSettingEntry>, ApplicationError> {
        self.database_manager
            .with_connection(|connection| SqliteSettingsRepository::list_all(connection))
    }

    pub fn load_retention_days_or_default(&self) -> Result<u64, ApplicationError> {
        let Some(entry) = self.get_setting(RETENTION_DAYS_SETTING_KEY)? else {
            return Ok(DEFAULT_RETENTION_DAYS);
        };
        let days = entry.value.as_u64().ok_or_else(|| {
            ApplicationError::Internal("Corrupted retention days setting".to_string())
        })?;
        validate_retention_days(days)
    }
}

fn validate_setting_key(raw_key: &str) -> Result<String, ApplicationError> {
    let normalized = raw_key.trim();
    if normalized.is_empty() || normalized.len() > 64 {
        return Err(ApplicationError::Validation(
            "Setting key length must be between 1 and 64 characters".to_string(),
        ));
    }
    let is_valid = normalized
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if !is_valid {
        return Err(ApplicationError::Validation(
            "Setting key must contain ASCII letters, digits, underscores, hyphens, or dots only"
                .to_string(),
        ));
    }
    Ok(normalized.to_string())
}

fn validate_known_setting_value(
    key: &str,
    value: &serde_json::Value,
) -> Result<(), ApplicationError> {
    match key {
        RETENTION_DAYS_SETTING_KEY => {
            let days = value.as_u64().ok_or_else(|| {
                ApplicationError::Validation(
                    "Retention days setting must be a positive integer".to_string(),
                )
            })?;
            validate_retention_days(days)?;
        }
        AUTO_UPDATE_CHECK_SETTING_KEY => {
            if !value.is_boolean() {
                return Err(ApplicationError::Validation(
                    "Auto update check setting must be a boolean".to_string(),
                ));
            }
        }
        WORKSPACE_SELECTION_SETTING_KEY => {
            let object = value.as_object().ok_or_else(|| {
                ApplicationError::Validation(
                    "Workspace selection setting must be a JSON object".to_string(),
                )
            })?;
            for field_name in [
                "activeWorkspaceId",
                "activeProjectId",
                "activeEnvironmentId",
            ] {
                if let Some(field_value) = object.get(field_name)
                    && !field_value.is_null()
                {
                    let identifier = field_value.as_str().ok_or_else(|| {
                        ApplicationError::Validation(format!(
                            "{field_name} must be a UUID string or null"
                        ))
                    })?;
                    validate_uuid(identifier, field_name)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
