use uuid::Uuid;

use crate::domain::environment::{Environment, EnvironmentKind};
use crate::domain::errors::ApplicationError;
use crate::domain::project::{CreateProjectInput, Project, UpdateProjectInput};
use crate::domain::validation::{
    current_utc_timestamp, derive_slug_or_validate, validate_description, validate_name,
    validate_slug, validate_uuid,
};
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::database::environment_repository::SqliteEnvironmentRepository;
use crate::infrastructure::database::project_repository::SqliteProjectRepository;
use crate::infrastructure::database::workspace_repository::SqliteWorkspaceRepository;

/// Orchestrates project lifecycle and automatic initialization of default environments.
#[derive(Clone)]
pub struct ProjectService {
    database_manager: DatabaseConnectionManager,
}

impl ProjectService {
    pub fn new(database_manager: DatabaseConnectionManager) -> Self {
        Self { database_manager }
    }

    /// Creates a project and initializes Local (default), Development, Testing, and Staging environments atomically.
    pub fn create_project(&self, input: CreateProjectInput) -> Result<Project, ApplicationError> {
        let workspace_id = validate_uuid(&input.workspace_id, "workspaceId")?;
        let name = validate_name(&input.name)?;
        let slug = derive_slug_or_validate(input.slug.as_deref(), &name)?;
        let description = validate_description(input.description.as_deref().unwrap_or_default())?;
        let now = current_utc_timestamp();

        let project = Project {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.clone(),
            name,
            slug,
            description,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        self.database_manager.with_transaction(|transaction| {
            if SqliteWorkspaceRepository::find_by_id(transaction, &workspace_id)?.is_none() {
                return Err(ApplicationError::NotFound("Workspace".to_string()));
            }

            SqliteProjectRepository::insert(transaction, &project)?;

            for kind in EnvironmentKind::STANDARD_DEFAULTS {
                let environment = Environment {
                    id: Uuid::new_v4().to_string(),
                    project_id: project.id.clone(),
                    name: kind.default_display_name().to_string(),
                    slug: kind.as_str().to_string(),
                    kind,
                    is_default: kind == EnvironmentKind::Local,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                SqliteEnvironmentRepository::insert(transaction, &environment)?;
            }

            Ok(project)
        })
    }

    pub fn list_projects_by_workspace(
        &self,
        raw_workspace_id: &str,
    ) -> Result<Vec<Project>, ApplicationError> {
        let workspace_id = validate_uuid(raw_workspace_id, "workspaceId")?;
        self.database_manager.with_connection(|connection| {
            SqliteProjectRepository::list_by_workspace(connection, &workspace_id)
        })
    }

    pub fn get_project(&self, raw_project_id: &str) -> Result<Project, ApplicationError> {
        let project_id = validate_uuid(raw_project_id, "projectId")?;
        self.database_manager.with_connection(|connection| {
            SqliteProjectRepository::find_by_id(connection, &project_id)?
                .ok_or_else(|| ApplicationError::NotFound("Project".to_string()))
        })
    }

    pub fn update_project(&self, input: UpdateProjectInput) -> Result<Project, ApplicationError> {
        let project_id = validate_uuid(&input.id, "projectId")?;
        let name = validate_name(&input.name)?;
        let slug = validate_slug(&input.slug)?;
        let description = validate_description(input.description.as_deref().unwrap_or_default())?;
        let now = current_utc_timestamp();

        self.database_manager.with_connection(|connection| {
            let existing = SqliteProjectRepository::find_by_id(connection, &project_id)?
                .ok_or_else(|| ApplicationError::NotFound("Project".to_string()))?;

            let updated = Project {
                id: existing.id,
                workspace_id: existing.workspace_id,
                name,
                slug,
                description,
                created_at: existing.created_at,
                updated_at: now,
            };

            let did_update = SqliteProjectRepository::update(connection, &updated)?;
            if !did_update {
                return Err(ApplicationError::NotFound("Project".to_string()));
            }
            Ok(updated)
        })
    }

    pub fn delete_project(&self, raw_project_id: &str) -> Result<(), ApplicationError> {
        let project_id = validate_uuid(raw_project_id, "projectId")?;
        self.database_manager.with_connection(|connection| {
            let did_delete = SqliteProjectRepository::delete_by_id(connection, &project_id)?;
            if !did_delete {
                return Err(ApplicationError::NotFound("Project".to_string()));
            }
            Ok(())
        })
    }
}
