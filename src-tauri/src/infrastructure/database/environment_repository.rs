use rusqlite::{Connection, Row, params};

use crate::domain::environment::{Environment, EnvironmentKind};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::database::sqlite_helpers::{map_sqlite_write_error, query_optional_row};

/// SQLite persistence operations for project-scoped environments.
pub struct SqliteEnvironmentRepository;

impl SqliteEnvironmentRepository {
    pub fn insert(
        connection: &Connection,
        environment: &Environment,
    ) -> Result<(), ApplicationError> {
        let is_default_flag: i64 = if environment.is_default { 1 } else { 0 };
        connection
            .execute(
                "INSERT INTO environments (id, project_id, name, slug, kind, is_default, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    environment.id,
                    environment.project_id,
                    environment.name,
                    environment.slug,
                    environment.kind.as_str(),
                    is_default_flag,
                    environment.created_at,
                    environment.updated_at,
                ],
            )
            .map_err(|error| {
                map_sqlite_write_error(error, "Environment slug already exists in this project")
            })?;
        Ok(())
    }

    pub fn find_by_id(
        connection: &Connection,
        environment_id: &str,
    ) -> Result<Option<Environment>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, project_id, name, slug, kind, is_default, created_at, updated_at
             FROM environments
             WHERE id = ?1",
        )?;
        query_optional_row(&mut statement, params![environment_id], map_environment_row)
    }

    pub fn list_by_project(
        connection: &Connection,
        project_id: &str,
    ) -> Result<Vec<Environment>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, project_id, name, slug, kind, is_default, created_at, updated_at
             FROM environments
             WHERE project_id = ?1
             ORDER BY is_default DESC, created_at ASC, name ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_environment_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ApplicationError::from)
    }

    pub fn set_default_for_project(
        connection: &Connection,
        project_id: &str,
        environment_id: &str,
        updated_at: &str,
    ) -> Result<bool, ApplicationError> {
        connection.execute(
            "UPDATE environments SET is_default = 0, updated_at = ?2 WHERE project_id = ?1",
            params![project_id, updated_at],
        )?;
        let affected = connection.execute(
            "UPDATE environments SET is_default = 1, updated_at = ?3 WHERE id = ?1 AND project_id = ?2",
            params![environment_id, project_id, updated_at],
        )?;
        Ok(affected > 0)
    }
}

fn map_environment_row(row: &Row<'_>) -> rusqlite::Result<Environment> {
    let raw_kind: String = row.get(4)?;
    let kind = EnvironmentKind::parse(&raw_kind).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let is_default_flag: i64 = row.get(5)?;

    Ok(Environment {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        slug: row.get(3)?,
        kind,
        is_default: is_default_flag == 1,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}
