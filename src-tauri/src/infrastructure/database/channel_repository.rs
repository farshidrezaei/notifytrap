use rusqlite::{Connection, Row, params};

use crate::domain::channel::{
    ChannelChaosConfiguration, ChannelRecord, ChannelType, WebhookMockConfiguration,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::database::sqlite_helpers::{map_sqlite_write_error, query_optional_row};

/// SQLite persistence operations for isolated capture channels.
pub struct SqliteChannelRepository;

impl SqliteChannelRepository {
    pub fn insert(
        connection: &Connection,
        channel: &ChannelRecord,
    ) -> Result<(), ApplicationError> {
        let is_enabled_flag: i64 = if channel.is_enabled { 1 } else { 0 };
        let webhook_mock_json = serde_json::to_string(&channel.webhook_mock).map_err(|error| {
            ApplicationError::Internal(format!("Failed to serialize webhook mock rule: {error}"))
        })?;
        let chaos_rule_json = serde_json::to_string(&channel.chaos_rule).map_err(|error| {
            ApplicationError::Internal(format!("Failed to serialize chaos rule: {error}"))
        })?;

        connection
            .execute(
                "INSERT INTO channels (
                    id, workspace_id, project_id, environment_id, name, channel_type,
                    is_enabled, credential_identifier, credential_hash, encrypted_credential,
                    webhook_mock_json, chaos_rule_json, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    channel.id,
                    channel.workspace_id,
                    channel.project_id,
                    channel.environment_id,
                    channel.name,
                    channel.channel_type.as_str(),
                    is_enabled_flag,
                    channel.credential_identifier,
                    channel.credential_hash,
                    channel.encrypted_credential,
                    webhook_mock_json,
                    chaos_rule_json,
                    channel.created_at,
                    channel.updated_at,
                ],
            )
            .map_err(|error| {
                map_sqlite_write_error(error, "Channel credential identifier already exists")
            })?;
        Ok(())
    }

    pub fn find_by_id(
        connection: &Connection,
        channel_id: &str,
    ) -> Result<Option<ChannelRecord>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, workspace_id, project_id, environment_id, name, channel_type,
                    is_enabled, credential_identifier, credential_hash, encrypted_credential,
                    webhook_mock_json, chaos_rule_json, created_at, updated_at
             FROM channels
             WHERE id = ?1",
        )?;
        query_optional_row(&mut statement, params![channel_id], map_channel_row)
    }

    pub fn find_by_credential_identifier(
        connection: &Connection,
        credential_identifier: &str,
    ) -> Result<Option<ChannelRecord>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, workspace_id, project_id, environment_id, name, channel_type,
                    is_enabled, credential_identifier, credential_hash, encrypted_credential,
                    webhook_mock_json, chaos_rule_json, created_at, updated_at
             FROM channels
             WHERE credential_identifier = ?1",
        )?;
        query_optional_row(
            &mut statement,
            params![credential_identifier],
            map_channel_row,
        )
    }

    pub fn list_by_scope(
        connection: &Connection,
        workspace_id: &str,
        project_id: &str,
        environment_id: &str,
    ) -> Result<Vec<ChannelRecord>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, workspace_id, project_id, environment_id, name, channel_type,
                    is_enabled, credential_identifier, credential_hash, encrypted_credential,
                    webhook_mock_json, chaos_rule_json, created_at, updated_at
             FROM channels
             WHERE workspace_id = ?1 AND project_id = ?2 AND environment_id = ?3
             ORDER BY created_at ASC, name ASC",
        )?;
        let rows = statement.query_map(
            params![workspace_id, project_id, environment_id],
            map_channel_row,
        )?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ApplicationError::from)
    }

    pub fn set_enabled(
        connection: &Connection,
        channel_id: &str,
        is_enabled: bool,
        updated_at: &str,
    ) -> Result<bool, ApplicationError> {
        let flag: i64 = if is_enabled { 1 } else { 0 };
        let affected = connection.execute(
            "UPDATE channels SET is_enabled = ?2, updated_at = ?3 WHERE id = ?1",
            params![channel_id, flag, updated_at],
        )?;
        Ok(affected > 0)
    }

    pub fn update_rotated_credential(
        connection: &Connection,
        channel_id: &str,
        credential_hash: &str,
        encrypted_credential: &str,
        updated_at: &str,
    ) -> Result<bool, ApplicationError> {
        let affected = connection.execute(
            "UPDATE channels
             SET credential_hash = ?2, encrypted_credential = ?3, updated_at = ?4
             WHERE id = ?1",
            params![
                channel_id,
                credential_hash,
                encrypted_credential,
                updated_at
            ],
        )?;
        Ok(affected > 0)
    }

    pub fn update_rules(
        connection: &Connection,
        channel_id: &str,
        webhook_mock: &WebhookMockConfiguration,
        chaos_rule: &ChannelChaosConfiguration,
        updated_at: &str,
    ) -> Result<bool, ApplicationError> {
        let webhook_mock_json = serde_json::to_string(webhook_mock).map_err(|error| {
            ApplicationError::Internal(format!("Failed to serialize webhook mock rule: {error}"))
        })?;
        let chaos_rule_json = serde_json::to_string(chaos_rule).map_err(|error| {
            ApplicationError::Internal(format!("Failed to serialize chaos rule: {error}"))
        })?;
        let affected = connection.execute(
            "UPDATE channels
             SET webhook_mock_json = ?2, chaos_rule_json = ?3, updated_at = ?4
             WHERE id = ?1",
            params![channel_id, webhook_mock_json, chaos_rule_json, updated_at],
        )?;
        Ok(affected > 0)
    }

    pub fn delete_by_id(
        connection: &Connection,
        channel_id: &str,
    ) -> Result<bool, ApplicationError> {
        let affected =
            connection.execute("DELETE FROM channels WHERE id = ?1", params![channel_id])?;
        Ok(affected > 0)
    }
}

fn map_channel_row(row: &Row<'_>) -> rusqlite::Result<ChannelRecord> {
    let raw_type: String = row.get(5)?;
    let channel_type = ChannelType::parse(&raw_type).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let is_enabled_flag: i64 = row.get(6)?;
    let raw_mock_json: String = row.get(10)?;
    let raw_chaos_json: String = row.get(11)?;

    let webhook_mock = if raw_mock_json.trim().is_empty() {
        WebhookMockConfiguration::default()
    } else {
        serde_json::from_str(&raw_mock_json).unwrap_or_default()
    };
    let chaos_rule = if raw_chaos_json.trim().is_empty() {
        ChannelChaosConfiguration::default()
    } else {
        serde_json::from_str(&raw_chaos_json).unwrap_or_default()
    };

    Ok(ChannelRecord {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        project_id: row.get(2)?,
        environment_id: row.get(3)?,
        name: row.get(4)?,
        channel_type,
        is_enabled: is_enabled_flag == 1,
        credential_identifier: row.get(7)?,
        credential_hash: row.get(8)?,
        encrypted_credential: row.get(9)?,
        webhook_mock,
        chaos_rule,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}
