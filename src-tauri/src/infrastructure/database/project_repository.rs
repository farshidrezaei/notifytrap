use rusqlite::{Connection, Row, params};

use crate::domain::errors::ApplicationError;
use crate::domain::project::Project;
use crate::infrastructure::database::sqlite_helpers::{map_sqlite_write_error, query_optional_row};

/// SQLite persistence operations for workspace-scoped projects.
pub struct SqliteProjectRepository;

impl SqliteProjectRepository {
    pub fn insert(connection: &Connection, project: &Project) -> Result<(), ApplicationError> {
        connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name, slug, description, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    project.id,
                    project.workspace_id,
                    project.name,
                    project.slug,
                    project.description,
                    project.created_at,
                    project.updated_at,
                ],
            )
            .map_err(|error| {
                map_sqlite_write_error(error, "Project slug already exists in this workspace")
            })?;
        Ok(())
    }

    pub fn find_by_id(
        connection: &Connection,
        project_id: &str,
    ) -> Result<Option<Project>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, workspace_id, name, slug, description, created_at, updated_at
             FROM projects
             WHERE id = ?1",
        )?;
        query_optional_row(&mut statement, params![project_id], map_project_row)
    }

    pub fn list_by_workspace(
        connection: &Connection,
        workspace_id: &str,
    ) -> Result<Vec<Project>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, workspace_id, name, slug, description, created_at, updated_at
             FROM projects
             WHERE workspace_id = ?1
             ORDER BY created_at ASC, name ASC",
        )?;
        let rows = statement.query_map(params![workspace_id], map_project_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ApplicationError::from)
    }

    pub fn update(connection: &Connection, project: &Project) -> Result<bool, ApplicationError> {
        let affected = connection
            .execute(
                "UPDATE projects
                 SET name = ?2, slug = ?3, description = ?4, updated_at = ?5
                 WHERE id = ?1",
                params![
                    project.id,
                    project.name,
                    project.slug,
                    project.description,
                    project.updated_at,
                ],
            )
            .map_err(|error| {
                map_sqlite_write_error(error, "Project slug already exists in this workspace")
            })?;
        Ok(affected > 0)
    }

    pub fn delete_by_id(
        connection: &Connection,
        project_id: &str,
    ) -> Result<bool, ApplicationError> {
        let affected =
            connection.execute("DELETE FROM projects WHERE id = ?1", params![project_id])?;
        Ok(affected > 0)
    }
}

fn map_project_row(row: &Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        slug: row.get(3)?,
        description: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
