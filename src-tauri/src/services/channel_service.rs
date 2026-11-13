use std::sync::Arc;

use uuid::Uuid;

use crate::domain::channel::{
    ChannelChaosConfiguration, ChannelCredentialReveal, ChannelRecord, ChannelSummary, ChannelType,
    CreateChannelInput, CreatedChannelResult, UpdateChannelRulesInput, WebhookMockConfiguration,
};
use crate::domain::errors::ApplicationError;
use crate::domain::forwarding::ChannelForwardingConfiguration;
use crate::domain::validation::{
    current_utc_timestamp, validate_bounded_text, validate_name, validate_uuid,
};
use crate::infrastructure::database::channel_repository::SqliteChannelRepository;
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::database::environment_repository::SqliteEnvironmentRepository;
use crate::infrastructure::database::project_repository::SqliteProjectRepository;
use crate::infrastructure::security::credential_crypto::{
    decrypt_secret, generate_channel_credential, parse_composite_channel_key,
    verify_secret_constant_time,
};
use crate::infrastructure::security::secret_key_provider::SecretKeyProvider;

/// Orchestrates channel creation, scope verification, secret encryption, rotation, and authentication.
#[derive(Clone)]
pub struct ChannelService {
    database_manager: DatabaseConnectionManager,
    secret_key_provider: Arc<dyn SecretKeyProvider>,
}

impl ChannelService {
    pub fn new(
        database_manager: DatabaseConnectionManager,
        secret_key_provider: Arc<dyn SecretKeyProvider>,
    ) -> Self {
        Self {
            database_manager,
            secret_key_provider,
        }
    }

    pub fn create_channel(
        &self,
        input: CreateChannelInput,
    ) -> Result<CreatedChannelResult, ApplicationError> {
        let workspace_id = validate_uuid(&input.workspace_id, "workspaceId")?;
        let project_id = validate_uuid(&input.project_id, "projectId")?;
        let environment_id = validate_uuid(&input.environment_id, "environmentId")?;
        let name = validate_name(&input.name)?;
        let master_key = self.secret_key_provider.get_or_create_master_key()?;
        let generated = generate_channel_credential(
            input.channel_type,
            input.credential_identifier.as_deref(),
            &master_key,
        )?;
        let now = current_utc_timestamp();

        let record = ChannelRecord {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.clone(),
            project_id: project_id.clone(),
            environment_id: environment_id.clone(),
            name,
            channel_type: input.channel_type,
            is_enabled: input.is_enabled.unwrap_or(true),
            credential_identifier: generated.credential_identifier.clone(),
            credential_hash: generated.credential_hash,
            encrypted_credential: generated.encrypted_credential,
            webhook_mock: WebhookMockConfiguration::default(),
            chaos_rule: ChannelChaosConfiguration::default(),
            forwarding_rule: ChannelForwardingConfiguration::default(),
            created_at: now.clone(),
            updated_at: now,
        };

        self.database_manager.with_transaction(|transaction| {
            verify_scope_hierarchy(transaction, &workspace_id, &project_id, &environment_id)?;
            SqliteChannelRepository::insert(transaction, &record)?;
            Ok(())
        })?;

        Ok(CreatedChannelResult {
            channel: record.to_summary(),
            credential: ChannelCredentialReveal {
                channel_id: record.id,
                credential_identifier: generated.credential_identifier,
                secret_token: generated.secret_token,
                composite_key: generated.composite_key,
            },
        })
    }

    pub fn list_channels_by_scope(
        &self,
        raw_workspace_id: &str,
        raw_project_id: &str,
        raw_environment_id: &str,
    ) -> Result<Vec<ChannelSummary>, ApplicationError> {
        let workspace_id = validate_uuid(raw_workspace_id, "workspaceId")?;
        let project_id = validate_uuid(raw_project_id, "projectId")?;
        let environment_id = validate_uuid(raw_environment_id, "environmentId")?;

        self.database_manager.with_connection(|connection| {
            verify_scope_hierarchy(connection, &workspace_id, &project_id, &environment_id)?;
            let records = SqliteChannelRepository::list_by_scope(
                connection,
                &workspace_id,
                &project_id,
                &environment_id,
            )?;
            Ok(records
                .into_iter()
                .map(|record| record.to_summary())
                .collect())
        })
    }

    pub fn set_channel_enabled(
        &self,
        raw_channel_id: &str,
        is_enabled: bool,
    ) -> Result<ChannelSummary, ApplicationError> {
        let channel_id = validate_uuid(raw_channel_id, "channelId")?;
        let now = current_utc_timestamp();

        self.database_manager.with_connection(|connection| {
            let did_update =
                SqliteChannelRepository::set_enabled(connection, &channel_id, is_enabled, &now)?;
            if !did_update {
                return Err(ApplicationError::NotFound("Channel".to_string()));
            }
            let updated = SqliteChannelRepository::find_by_id(connection, &channel_id)?
                .ok_or_else(|| ApplicationError::NotFound("Channel".to_string()))?;
            Ok(updated.to_summary())
        })
    }

    pub fn update_channel_rules(
        &self,
        input: UpdateChannelRulesInput,
    ) -> Result<ChannelSummary, ApplicationError> {
        let channel_id = validate_uuid(&input.channel_id, "channelId")?;
        let now = current_utc_timestamp();

        let existing = self.database_manager.with_connection(|connection| {
            SqliteChannelRepository::find_by_id(connection, &channel_id)?
                .ok_or_else(|| ApplicationError::NotFound("Channel".to_string()))
        })?;

        let validated_mock = validate_webhook_mock(existing.channel_type, input.webhook_mock)?;
        let validated_chaos = validate_chaos_rule(input.chaos_rule)?;
        let validated_forwarding = validate_forwarding_rule(input.forwarding_rule)?;

        self.database_manager.with_connection(|connection| {
            let did_update = SqliteChannelRepository::update_rules(
                connection,
                &channel_id,
                &validated_mock,
                &validated_chaos,
                &validated_forwarding,
                &now,
            )?;
            if !did_update {
                return Err(ApplicationError::NotFound("Channel".to_string()));
            }
            let updated = SqliteChannelRepository::find_by_id(connection, &channel_id)?
                .ok_or_else(|| ApplicationError::NotFound("Channel".to_string()))?;
            Ok(updated.to_summary())
        })
    }

    pub fn reveal_channel_credential(
        &self,
        raw_channel_id: &str,
    ) -> Result<ChannelCredentialReveal, ApplicationError> {
        let channel_id = validate_uuid(raw_channel_id, "channelId")?;
        let master_key = self.secret_key_provider.get_or_create_master_key()?;

        let record = self.database_manager.with_connection(|connection| {
            SqliteChannelRepository::find_by_id(connection, &channel_id)?
                .ok_or_else(|| ApplicationError::NotFound("Channel".to_string()))
        })?;

        let secret_token = decrypt_secret(&master_key, &record.encrypted_credential)?;
        let composite_key = format!("{}.{secret_token}", record.credential_identifier);

        Ok(ChannelCredentialReveal {
            channel_id: record.id,
            credential_identifier: record.credential_identifier,
            secret_token,
            composite_key,
        })
    }

    pub fn rotate_channel_credential(
        &self,
        raw_channel_id: &str,
    ) -> Result<ChannelCredentialReveal, ApplicationError> {
        let channel_id = validate_uuid(raw_channel_id, "channelId")?;
        let master_key = self.secret_key_provider.get_or_create_master_key()?;
        let now = current_utc_timestamp();

        let existing = self.database_manager.with_connection(|connection| {
            SqliteChannelRepository::find_by_id(connection, &channel_id)?
                .ok_or_else(|| ApplicationError::NotFound("Channel".to_string()))
        })?;

        let generated = generate_channel_credential(
            existing.channel_type,
            Some(&existing.credential_identifier),
            &master_key,
        )?;

        self.database_manager.with_connection(|connection| {
            let did_update = SqliteChannelRepository::update_rotated_credential(
                connection,
                &channel_id,
                &generated.credential_hash,
                &generated.encrypted_credential,
                &now,
            )?;
            if !did_update {
                return Err(ApplicationError::NotFound("Channel".to_string()));
            }
            Ok(())
        })?;

        Ok(ChannelCredentialReveal {
            channel_id: existing.id,
            credential_identifier: generated.credential_identifier,
            secret_token: generated.secret_token,
            composite_key: generated.composite_key,
        })
    }

    /// Authenticates an incoming `credential_identifier` and `secret` in constant time.
    pub fn authenticate_identifier_and_secret(
        &self,
        credential_identifier: &str,
        secret_token: &str,
    ) -> Result<ChannelSummary, ApplicationError> {
        let trimmed_identifier = credential_identifier.trim();
        let trimmed_secret = secret_token.trim();
        if trimmed_identifier.is_empty() || trimmed_secret.is_empty() {
            return Err(ApplicationError::Unauthorized);
        }

        let maybe_record = self.database_manager.with_connection(|connection| {
            SqliteChannelRepository::find_by_credential_identifier(connection, trimmed_identifier)
        })?;

        let Some(record) = maybe_record else {
            return Err(ApplicationError::Unauthorized);
        };

        if !record.is_enabled {
            return Err(ApplicationError::Unauthorized);
        }

        if !verify_secret_constant_time(trimmed_secret, &record.credential_hash) {
            return Err(ApplicationError::Unauthorized);
        }

        Ok(record.to_summary())
    }

    /// Authenticates an incoming `<credential_identifier>.<secret_token>` HTTP key.
    pub fn authenticate_composite_key(
        &self,
        composite_key: &str,
    ) -> Result<ChannelSummary, ApplicationError> {
        let Some((identifier, secret)) = parse_composite_channel_key(composite_key) else {
            return Err(ApplicationError::Unauthorized);
        };
        self.authenticate_identifier_and_secret(identifier, secret)
    }

    pub fn delete_channel(&self, raw_channel_id: &str) -> Result<(), ApplicationError> {
        let channel_id = validate_uuid(raw_channel_id, "channelId")?;
        self.database_manager.with_connection(|connection| {
            let did_delete = SqliteChannelRepository::delete_by_id(connection, &channel_id)?;
            if !did_delete {
                return Err(ApplicationError::NotFound("Channel".to_string()));
            }
            Ok(())
        })
    }
}

fn verify_scope_hierarchy(
    connection: &rusqlite::Connection,
    workspace_id: &str,
    project_id: &str,
    environment_id: &str,
) -> Result<(), ApplicationError> {
    let project = SqliteProjectRepository::find_by_id(connection, project_id)?
        .ok_or_else(|| ApplicationError::NotFound("Project".to_string()))?;

    if project.workspace_id != workspace_id {
        return Err(ApplicationError::Validation(
            "Project does not belong to the specified workspace".to_string(),
        ));
    }

    let environment = SqliteEnvironmentRepository::find_by_id(connection, environment_id)?
        .ok_or_else(|| ApplicationError::NotFound("Environment".to_string()))?;

    if environment.project_id != project_id {
        return Err(ApplicationError::Validation(
            "Environment does not belong to the specified project".to_string(),
        ));
    }

    Ok(())
}

const MAXIMUM_MOCK_HEADERS: usize = 32;
const MAXIMUM_MOCK_BODY_BYTES: usize = 65_536;
const MAXIMUM_SIMULATED_DELAY_MILLISECONDS: u64 = 10_000;

fn validate_webhook_mock(
    channel_type: ChannelType,
    mut configuration: WebhookMockConfiguration,
) -> Result<WebhookMockConfiguration, ApplicationError> {
    if channel_type != ChannelType::Webhook && configuration.is_enabled {
        return Err(ApplicationError::Validation(
            "Custom webhook mock responses are only supported on webhook channels".to_string(),
        ));
    }
    if !(100..=599).contains(&configuration.status_code) {
        return Err(ApplicationError::Validation(
            "Webhook mock HTTP status code must be between 100 and 599".to_string(),
        ));
    }
    if configuration.delay_milliseconds > MAXIMUM_SIMULATED_DELAY_MILLISECONDS {
        return Err(ApplicationError::Validation(format!(
            "Webhook mock delay must not exceed {MAXIMUM_SIMULATED_DELAY_MILLISECONDS} ms"
        )));
    }
    if configuration.body.len() > MAXIMUM_MOCK_BODY_BYTES {
        return Err(ApplicationError::Validation(format!(
            "Webhook mock body must not exceed {MAXIMUM_MOCK_BODY_BYTES} bytes"
        )));
    }
    if configuration.headers.len() > MAXIMUM_MOCK_HEADERS {
        return Err(ApplicationError::Validation(format!(
            "Webhook mock headers must not exceed {MAXIMUM_MOCK_HEADERS} entries"
        )));
    }

    let mut sanitized_headers = std::collections::BTreeMap::new();
    for (raw_name, raw_value) in configuration.headers {
        let name = validate_bounded_text(&raw_name, "mockHeaderName", 1, 128, false)?;
        if !name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        }) {
            return Err(ApplicationError::Validation(format!(
                "Invalid mock response header name: {name}"
            )));
        }
        let value = validate_bounded_text(&raw_value, "mockHeaderValue", 0, 1024, false)?;
        sanitized_headers.insert(name, value);
    }
    configuration.headers = sanitized_headers;
    Ok(configuration)
}

fn validate_chaos_rule(
    mut configuration: ChannelChaosConfiguration,
) -> Result<ChannelChaosConfiguration, ApplicationError> {
    if configuration.latency_milliseconds > MAXIMUM_SIMULATED_DELAY_MILLISECONDS {
        return Err(ApplicationError::Validation(format!(
            "Chaos latency must not exceed {MAXIMUM_SIMULATED_DELAY_MILLISECONDS} ms"
        )));
    }
    if configuration.failure_rate_percent > 100 {
        return Err(ApplicationError::Validation(
            "Chaos failure rate must be between 0 and 100 percent".to_string(),
        ));
    }
    if !(400..=599).contains(&configuration.http_error_status_code) {
        return Err(ApplicationError::Validation(
            "Chaos HTTP error status code must be between 400 and 599".to_string(),
        ));
    }
    if !(400..=599).contains(&configuration.smtp_error_code) {
        return Err(ApplicationError::Validation(
            "Chaos SMTP error code must be between 400 and 599".to_string(),
        ));
    }
    configuration.error_message = validate_bounded_text(
        &configuration.error_message,
        "chaosErrorMessage",
        1,
        240,
        false,
    )?;
    Ok(configuration)
}

fn validate_forwarding_rule(
    mut configuration: ChannelForwardingConfiguration,
) -> Result<ChannelForwardingConfiguration, ApplicationError> {
    if !configuration.is_enabled {
        return Ok(configuration);
    }
    let target = configuration.target_url.trim();
    if target.is_empty() {
        return Err(ApplicationError::Validation(
            "Forwarding target URL must not be empty when enabled".to_string(),
        ));
    }
    if !target.starts_with("http://") && !target.starts_with("https://") {
        return Err(ApplicationError::Validation(
            "Forwarding target URL must start with http:// or https://".to_string(),
        ));
    }
    if configuration.timeout_seconds == 0 || configuration.timeout_seconds > 60 {
        return Err(ApplicationError::Validation(
            "Forwarding timeout must be between 1 and 60 seconds".to_string(),
        ));
    }
    if configuration.custom_headers.len() > MAXIMUM_MOCK_HEADERS {
        return Err(ApplicationError::Validation(format!(
            "Forwarding custom headers must not exceed {MAXIMUM_MOCK_HEADERS} entries"
        )));
    }

    let mut sanitized_headers = std::collections::BTreeMap::new();
    for (raw_name, raw_value) in configuration.custom_headers {
        let name = validate_bounded_text(&raw_name, "forwardingHeaderName", 1, 128, false)?;
        if !name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        }) {
            return Err(ApplicationError::Validation(format!(
                "Invalid forwarding header name: {name}"
            )));
        }
        let value = validate_bounded_text(&raw_value, "forwardingHeaderValue", 0, 1024, false)?;
        sanitized_headers.insert(name, value);
    }
    configuration.custom_headers = sanitized_headers;
    configuration.target_url = target.to_string();
    Ok(configuration)
}
