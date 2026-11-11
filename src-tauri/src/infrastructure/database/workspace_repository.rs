use rusqlite::{Connection, Row, params};

use crate::domain::errors::ApplicationError;
use crate::domain::workspace::Workspace;
use crate::infrastructure::database::sqlite_helpers::{map_sqlite_write_error, query_optional_row};

/// SQLite persistence operations for workspaces.
pub struct SqliteWorkspaceRepository;

impl SqliteWorkspaceRepository {
    pub fn insert(connection: &Connection, workspace: &Workspace) -> Result<(), ApplicationError> {
        connection
            .execute(
                "INSERT INTO workspaces (id, name, slug, description, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    workspace.id,
                    workspace.name,
                    workspace.slug,
                    workspace.description,
                    workspace.created_at,
                    workspace.updated_at,
                ],
            )
            .map_err(|error| {
                map_sqlite_write_error(error, "Workspace name or slug already exists")
            })?;
        Ok(())
    }

    pub fn find_by_id(
        connection: &Connection,
        workspace_id: &str,
    ) -> Result<Option<Workspace>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, name, slug, description, created_at, updated_at
             FROM workspaces
             WHERE id = ?1",
        )?;
        query_optional_row(&mut statement, params![workspace_id], map_workspace_row)
    }

    pub fn find_by_slug(
        connection: &Connection,
        slug: &str,
    ) -> Result<Option<Workspace>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, name, slug, description, created_at, updated_at
             FROM workspaces
             WHERE slug = ?1",
        )?;
        query_optional_row(&mut statement, params![slug], map_workspace_row)
    }

    pub fn list_all(connection: &Connection) -> Result<Vec<Workspace>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, name, slug, description, created_at, updated_at
             FROM workspaces
             ORDER BY created_at ASC, name ASC",
        )?;
        let rows = statement.query_map([], map_workspace_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ApplicationError::from)
    }

    pub fn update(
        connection: &Connection,
        workspace: &Workspace,
    ) -> Result<bool, ApplicationError> {
        let affected = connection
            .execute(
                "UPDATE workspaces
                 SET name = ?2, slug = ?3, description = ?4, updated_at = ?5
                 WHERE id = ?1",
                params![
                    workspace.id,
                    workspace.name,
                    workspace.slug,
                    workspace.description,
                    workspace.updated_at,
                ],
            )
            .map_err(|error| {
                map_sqlite_write_error(error, "Workspace name or slug already exists")
            })?;
        Ok(affected > 0)
    }

    pub fn delete_by_id(
        connection: &Connection,
        workspace_id: &str,
    ) -> Result<bool, ApplicationError> {
        let affected = connection.execute(
            "DELETE FROM workspaces WHERE id = ?1",
            params![workspace_id],
        )?;
        Ok(affected > 0)
    }
}

fn map_workspace_row(row: &Row<'_>) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        description: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}
