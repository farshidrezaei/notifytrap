use rusqlite::{Connection, Row, params};

use crate::domain::errors::ApplicationError;
use crate::domain::settings::ApplicationSettingEntry;
use crate::infrastructure::database::sqlite_helpers::query_optional_row;

/// SQLite persistence operations for typed application settings.
pub struct SqliteSettingsRepository;

impl SqliteSettingsRepository {
    pub fn upsert(
        connection: &Connection,
        entry: &ApplicationSettingEntry,
    ) -> Result<(), ApplicationError> {
        let value_json = serde_json::to_string(&entry.value).map_err(|error| {
            ApplicationError::Internal(format!("Failed to serialize setting value: {error}"))
        })?;

        connection.execute(
            "INSERT INTO application_settings (key, value_json, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET
               value_json = excluded.value_json,
               updated_at = excluded.updated_at",
            params![entry.key, value_json, entry.updated_at],
        )?;
        Ok(())
    }

    pub fn find_by_key(
        connection: &Connection,
        key: &str,
    ) -> Result<Option<ApplicationSettingEntry>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT key, value_json, updated_at FROM application_settings WHERE key = ?1",
        )?;
        query_optional_row(&mut statement, params![key], map_setting_row)
    }

    pub fn list_all(
        connection: &Connection,
    ) -> Result<Vec<ApplicationSettingEntry>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT key, value_json, updated_at FROM application_settings ORDER BY key ASC",
        )?;
        let rows = statement.query_map([], map_setting_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ApplicationError::from)
    }
}

fn map_setting_row(row: &Row<'_>) -> rusqlite::Result<ApplicationSettingEntry> {
    let raw_json: String = row.get(1)?;
    let value: serde_json::Value = serde_json::from_str(&raw_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(ApplicationSettingEntry {
        key: row.get(0)?,
        value,
        updated_at: row.get(2)?,
    })
}
