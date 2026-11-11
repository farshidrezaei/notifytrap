use std::sync::Arc;

use uuid::Uuid;

use crate::domain::channel::{
    ChannelCredentialReveal, ChannelRecord, ChannelSummary, CreateChannelInput,
    CreatedChannelResult,
};
use crate::domain::errors::ApplicationError;
use crate::domain::validation::{current_utc_timestamp, validate_name, validate_uuid};
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
