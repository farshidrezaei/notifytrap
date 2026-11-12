use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::domain::database_statistics::DatabaseStorageStatistics;
use crate::domain::errors::ApplicationError;
use crate::infrastructure::migrations;

/// Thread-safe SQLite database connection manager with foreign-key and WAL enforcement.
#[derive(Clone)]
pub struct DatabaseConnectionManager {
    connection: Arc<Mutex<Connection>>,
    database_path: Option<PathBuf>,
}

impl DatabaseConnectionManager {
    const BUSY_TIMEOUT_MILLISECONDS: u64 = 5000;

    /// Opens or creates a file-backed SQLite database, applies PRAGMAs, and runs pending migrations.
    pub fn open_file(path: &Path) -> Result<Self, ApplicationError> {
        let connection = Connection::open(path)?;
        Self::initialize_connection(connection, Some(path.to_path_buf()))
    }

    /// Opens an isolated in-memory SQLite database for deterministic tests.
    pub fn open_in_memory() -> Result<Self, ApplicationError> {
        let connection = Connection::open_in_memory()?;
        Self::initialize_connection(connection, None)
    }

    fn initialize_connection(
        mut connection: Connection,
        database_path: Option<PathBuf>,
    ) -> Result<Self, ApplicationError> {
        connection.busy_timeout(std::time::Duration::from_millis(
            Self::BUSY_TIMEOUT_MILLISECONDS,
        ))?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;

        migrations::apply_all_migrations(&mut connection)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            database_path,
        })
    }

    /// Executes a synchronous closure with exclusive access to the SQLite connection.
    pub fn with_connection<F, T>(&self, operation: F) -> Result<T, ApplicationError>
    where
        F: FnOnce(&mut Connection) -> Result<T, ApplicationError>,
    {
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| ApplicationError::Internal("Database mutex poisoned".to_string()))?;
        operation(&mut guard)
    }

    /// Executes a synchronous closure inside an atomic SQLite transaction.
    pub fn with_transaction<F, T>(&self, operation: F) -> Result<T, ApplicationError>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T, ApplicationError>,
    {
        self.with_connection(|connection| {
            let transaction = connection.transaction()?;
            let outcome = operation(&transaction)?;
            transaction.commit()?;
            Ok(outcome)
        })
    }

    /// Returns the filesystem path of the SQLite database if file-backed.
    pub fn database_path(&self) -> Option<&Path> {
        self.database_path.as_deref()
    }

    /// Calculates comprehensive storage and footprint metrics for the SQLite database.
    pub fn get_storage_statistics(&self) -> Result<DatabaseStorageStatistics, ApplicationError> {
        let (file_size_bytes, wal_size_bytes) = if let Some(path) = &self.database_path {
            let primary_size = std::fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0);
            let wal_path = PathBuf::from(format!("{}-wal", path.display()));
            let wal_size = std::fs::metadata(&wal_path).map(|metadata| metadata.len()).unwrap_or(0);
            (primary_size, wal_size)
        } else {
            let size = self.with_connection(|connection| {
                let page_count: i64 = connection.query_row("PRAGMA page_count;", [], |row| row.get(0))?;
                let page_size: i64 = connection.query_row("PRAGMA page_size;", [], |row| row.get(0))?;
                Ok((page_count.max(0) * page_size.max(0)) as u64)
            })?;
            (size, 0)
        };

        let (total_notifications_count, total_attachments_count, total_attachments_size_bytes) =
            self.with_connection(|connection| {
                let notifications_count: i64 = connection
                    .query_row("SELECT COUNT(*) FROM notifications;", [], |row| row.get(0))
                    .unwrap_or(0);

                let (attachments_count, attachments_size): (i64, i64) = connection
                    .query_row(
                        "SELECT COUNT(*), COALESCE(SUM(size_bytes), 0) FROM notification_attachments;",
                        [],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .unwrap_or((0, 0));

                Ok((
                    notifications_count.max(0) as u64,
                    attachments_count.max(0) as u64,
                    attachments_size.max(0) as u64,
                ))
            })?;

        Ok(DatabaseStorageStatistics {
            database_path: self.database_path.as_ref().map(|path| path.display().to_string()),
            file_size_bytes,
            wal_size_bytes,
            total_notifications_count,
            total_attachments_count,
            total_attachments_size_bytes,
        })
    }

    /// Executes VACUUM on the SQLite database to reclaim fragmented pages and compact storage.
    pub fn vacuum_database(&self) -> Result<(), ApplicationError> {
        self.with_connection(|connection| {
            connection.execute_batch("VACUUM;")?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DatabaseConnectionManager;

    #[test]
    fn enforces_foreign_keys_and_applies_initial_schema() {
        let manager = DatabaseConnectionManager::open_in_memory()
            .expect("in-memory database should initialize");

        let foreign_keys_enabled: i64 = manager
            .with_connection(|connection| {
                let value: i64 =
                    connection.query_row("PRAGMA foreign_keys;", [], |row| row.get(0))?;
                Ok(value)
            })
            .expect("pragma query should succeed");
        assert_eq!(foreign_keys_enabled, 1);

        let table_count: i64 = manager
            .with_connection(|connection| {
                let count: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('workspaces', 'projects', 'environments', 'channels', 'notifications', 'notification_attachments', 'application_settings', 'schema_migrations');",
                    [],
                    |row| row.get(0),
                )?;
                Ok(count)
            })
            .expect("table count query should succeed");
        assert_eq!(table_count, 8);
    }

    #[test]
    fn rejects_orphan_project_creation_via_foreign_key() {
        let manager = DatabaseConnectionManager::open_in_memory()
            .expect("in-memory database should initialize");

        let insert_result = manager.with_connection(|connection| {
            connection.execute(
                "INSERT INTO projects (id, workspace_id, name, slug, created_at, updated_at) VALUES ('p1', 'missing-workspace', 'Demo', 'demo', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )?;
            Ok(())
        });

        assert!(insert_result.is_err());
    }
}
