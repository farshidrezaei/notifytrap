use rusqlite::{ErrorCode, OptionalExtension};

use crate::domain::errors::ApplicationError;

/// Maps SQLite unique or foreign-key constraint errors to clean domain errors.
pub fn map_sqlite_write_error(error: rusqlite::Error, conflict_message: &str) -> ApplicationError {
    if let rusqlite::Error::SqliteFailure(sqlite_error, _) = &error
        && sqlite_error.code == ErrorCode::ConstraintViolation
    {
        return ApplicationError::Conflict(conflict_message.to_string());
    }
    ApplicationError::from(error)
}

/// Normalizes rusqlite `optional()` behavior for single-row lookups.
pub fn query_optional_row<T, P, F>(
    statement: &mut rusqlite::Statement<'_>,
    parameters: P,
    mapper: F,
) -> Result<Option<T>, ApplicationError>
where
    P: rusqlite::Params,
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
{
    statement
        .query_row(parameters, mapper)
        .optional()
        .map_err(ApplicationError::from)
}
