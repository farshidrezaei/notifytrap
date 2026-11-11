use uuid::Uuid;

use crate::domain::environment::{CreateEnvironmentInput, Environment};
use crate::domain::errors::ApplicationError;
use crate::domain::validation::{
    current_utc_timestamp, derive_slug_or_validate, validate_name, validate_uuid,
};
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::database::environment_repository::SqliteEnvironmentRepository;
use crate::infrastructure::database::project_repository::SqliteProjectRepository;

/// Orchestrates project-scoped environments and default selection guarantees.
#[derive(Clone)]
pub struct EnvironmentService {
    database_manager: DatabaseConnectionManager,
}

impl EnvironmentService {
    pub fn new(database_manager: DatabaseConnectionManager) -> Self {
        Self { database_manager }
    }

    pub fn list_environments_by_project(
        &self,
        raw_project_id: &str,
    ) -> Result<Vec<Environment>, ApplicationError> {
        let project_id = validate_uuid(raw_project_id, "projectId")?;
        self.database_manager.with_connection(|connection| {
            SqliteEnvironmentRepository::list_by_project(connection, &project_id)
        })
    }

    pub fn create_environment(
        &self,
        input: CreateEnvironmentInput,
    ) -> Result<Environment, ApplicationError> {
        let project_id = validate_uuid(&input.project_id, "projectId")?;
        let name = validate_name(&input.name)?;
        let slug = derive_slug_or_validate(input.slug.as_deref(), &name)?;
        let now = current_utc_timestamp();

        let environment = Environment {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.clone(),
            name,
            slug,
            kind: input.kind,
            is_default: input.is_default,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        self.database_manager.with_transaction(|transaction| {
            if SqliteProjectRepository::find_by_id(transaction, &project_id)?.is_none() {
                return Err(ApplicationError::NotFound("Project".to_string()));
            }

            SqliteEnvironmentRepository::insert(transaction, &environment)?;
            if environment.is_default {
                SqliteEnvironmentRepository::set_default_for_project(
                    transaction,
                    &project_id,
                    &environment.id,
                    &now,
                )?;
            }
            Ok(environment)
        })
    }

    pub fn set_default_environment(
        &self,
        raw_project_id: &str,
        raw_environment_id: &str,
    ) -> Result<Vec<Environment>, ApplicationError> {
        let project_id = validate_uuid(raw_project_id, "projectId")?;
        let environment_id = validate_uuid(raw_environment_id, "environmentId")?;
        let now = current_utc_timestamp();

        self.database_manager.with_transaction(|transaction| {
            let did_update = SqliteEnvironmentRepository::set_default_for_project(
                transaction,
                &project_id,
                &environment_id,
                &now,
            )?;
            if !did_update {
                return Err(ApplicationError::NotFound("Environment".to_string()));
            }
            SqliteEnvironmentRepository::list_by_project(transaction, &project_id)
        })
    }
}
