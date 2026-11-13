use rusqlite::Connection;

use crate::domain::errors::ApplicationError;

struct SqlMigration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[SqlMigration] = &[
    SqlMigration {
        version: 1,
        name: "0001_initial_schema",
        sql: include_str!("0001_initial_schema.sql"),
    },
    SqlMigration {
        version: 2,
        name: "0002_rename_tenant_to_workspace",
        sql: include_str!("0002_rename_tenant_to_workspace.sql"),
    },
    SqlMigration {
        version: 3,
        name: "0003_add_test_run_identifier",
        sql: include_str!("0003_add_test_run_identifier.sql"),
    },
    SqlMigration {
        version: 4,
        name: "0004_add_channel_rules",
        sql: include_str!("0004_add_channel_rules.sql"),
    },
    SqlMigration {
        version: 5,
        name: "0005_add_channel_forwarding_rules",
        sql: include_str!("0005_add_channel_forwarding_rules.sql"),
    },
];

/// Applies all unapplied SQLite migrations atomically in ascending version order.
pub fn apply_all_migrations(connection: &mut Connection) -> Result<(), ApplicationError> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );",
    )?;

    for migration in MIGRATIONS {
        let already_applied: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [migration.version],
            |row| row.get(0),
        )?;

        if already_applied {
            continue;
        }

        let transaction = connection.transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            rusqlite::params![migration.version, migration.name],
        )?;
        transaction.commit()?;
    }

    Ok(())
}

/// Returns the highest applied migration version.
pub fn current_migration_version(connection: &Connection) -> Result<i64, ApplicationError> {
    let version: i64 = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::{MIGRATIONS, apply_all_migrations, current_migration_version};
    use rusqlite::Connection;

    #[test]
    fn migrations_are_idempotent_and_tracked() {
        let mut connection = Connection::open_in_memory().expect("in-memory connection");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("pragma");

        apply_all_migrations(&mut connection).expect("first migration run");
        assert_eq!(current_migration_version(&connection).expect("version"), 5);

        apply_all_migrations(&mut connection).expect("second idempotent migration run");
        assert_eq!(current_migration_version(&connection).expect("version"), 5);
    }

    #[test]
    fn migration_v2_preserves_legacy_tenant_records_foreign_keys_and_settings() {
        let mut connection = Connection::open_in_memory().expect("in-memory connection");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("pragma");

        // Apply only v1 schema to simulate an existing local SQLite database.
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                );",
            )
            .expect("schema_migrations table");
        connection
            .execute_batch(MIGRATIONS[0].sql)
            .expect("apply v1 schema");
        connection
            .execute(
                "INSERT INTO schema_migrations (version, name) VALUES (1, '0001_initial_schema')",
                [],
            )
            .expect("track v1");

        connection
            .execute_batch(
                "INSERT INTO tenants (id, name, slug, description, created_at, updated_at)
                 VALUES ('w1', 'Acme', 'acme', 'Main', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO projects (id, tenant_id, name, slug, description, created_at, updated_at)
                 VALUES ('p1', 'w1', 'App', 'app', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO environments (id, project_id, name, slug, kind, is_default, created_at, updated_at)
                 VALUES ('e1', 'p1', 'Local', 'local', 'local', 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO application_settings (key, value_json, updated_at)
                 VALUES ('workspace.active_selection', '{\"activeTenantId\":\"w1\",\"activeProjectId\":\"p1\",\"activeEnvironmentId\":\"e1\"}', '2026-01-01T00:00:00Z');",
            )
            .expect("seed v1 rows");

        apply_all_migrations(&mut connection).expect("upgrade v1 -> v5");
        assert_eq!(current_migration_version(&connection).expect("version"), 5);

        let workspace_name: String = connection
            .query_row("SELECT name FROM workspaces WHERE id = 'w1'", [], |row| {
                row.get(0)
            })
            .expect("preserved workspace row");
        assert_eq!(workspace_name, "Acme");

        let project_workspace_id: String = connection
            .query_row(
                "SELECT workspace_id FROM projects WHERE id = 'p1'",
                [],
                |row| row.get(0),
            )
            .expect("migrated project.workspace_id");
        assert_eq!(project_workspace_id, "w1");

        let migrated_setting: String = connection
            .query_row(
                "SELECT json_extract(value_json, '$.activeWorkspaceId') FROM application_settings WHERE key = 'workspace.active_selection'",
                [],
                |row| row.get(0),
            )
            .expect("migrated activeWorkspaceId setting");
        assert_eq!(migrated_setting, "w1");

        let fk_violations: i64 = connection
            .query_row("PRAGMA foreign_key_check;", [], |row| row.get(0))
            .unwrap_or(0);
        assert_eq!(fk_violations, 0);
    }
}
