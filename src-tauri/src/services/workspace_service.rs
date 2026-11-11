use uuid::Uuid;

use crate::domain::errors::ApplicationError;
use crate::domain::validation::{
    current_utc_timestamp, derive_slug_or_validate, validate_description, validate_name,
    validate_slug, validate_uuid,
};
use crate::domain::workspace::{CreateWorkspaceInput, UpdateWorkspaceInput, Workspace};
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::database::workspace_repository::SqliteWorkspaceRepository;

/// Orchestrates workspace lifecycle and validation invariants.
#[derive(Clone)]
pub struct WorkspaceService {
    database_manager: DatabaseConnectionManager,
}

impl WorkspaceService {
    pub fn new(database_manager: DatabaseConnectionManager) -> Self {
        Self { database_manager }
    }

    pub fn create_workspace(
        &self,
        input: CreateWorkspaceInput,
    ) -> Result<Workspace, ApplicationError> {
        let name = validate_name(&input.name)?;
        let slug = derive_slug_or_validate(input.slug.as_deref(), &name)?;
        let description = validate_description(input.description.as_deref().unwrap_or_default())?;
        let now = current_utc_timestamp();

        let workspace = Workspace {
            id: Uuid::new_v4().to_string(),
            name,
            slug,
            description,
            created_at: now.clone(),
            updated_at: now,
        };

        self.database_manager.with_connection(|connection| {
            SqliteWorkspaceRepository::insert(connection, &workspace)
        })?;

        Ok(workspace)
    }

    pub fn list_workspaces(&self) -> Result<Vec<Workspace>, ApplicationError> {
        self.database_manager
            .with_connection(|connection| SqliteWorkspaceRepository::list_all(connection))
    }

    pub fn get_workspace(&self, raw_workspace_id: &str) -> Result<Workspace, ApplicationError> {
        let workspace_id = validate_uuid(raw_workspace_id, "workspaceId")?;
        self.database_manager.with_connection(|connection| {
            SqliteWorkspaceRepository::find_by_id(connection, &workspace_id)?
                .ok_or_else(|| ApplicationError::NotFound("Workspace".to_string()))
        })
    }

    pub fn update_workspace(
        &self,
        input: UpdateWorkspaceInput,
    ) -> Result<Workspace, ApplicationError> {
        let workspace_id = validate_uuid(&input.id, "workspaceId")?;
        let name = validate_name(&input.name)?;
        let slug = validate_slug(&input.slug)?;
        let description = validate_description(input.description.as_deref().unwrap_or_default())?;
        let now = current_utc_timestamp();

        self.database_manager.with_connection(|connection| {
            let existing = SqliteWorkspaceRepository::find_by_id(connection, &workspace_id)?
                .ok_or_else(|| ApplicationError::NotFound("Workspace".to_string()))?;

            let updated = Workspace {
                id: existing.id,
                name,
                slug,
                description,
                created_at: existing.created_at,
                updated_at: now,
            };

            let did_update = SqliteWorkspaceRepository::update(connection, &updated)?;
            if !did_update {
                return Err(ApplicationError::NotFound("Workspace".to_string()));
            }
            Ok(updated)
        })
    }

    pub fn delete_workspace(&self, raw_workspace_id: &str) -> Result<(), ApplicationError> {
        let workspace_id = validate_uuid(raw_workspace_id, "workspaceId")?;
        self.database_manager.with_connection(|connection| {
            let did_delete = SqliteWorkspaceRepository::delete_by_id(connection, &workspace_id)?;
            if !did_delete {
                return Err(ApplicationError::NotFound("Workspace".to_string()));
            }
            Ok(())
        })
    }
}
