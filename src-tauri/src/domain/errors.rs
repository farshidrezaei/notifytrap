use serde::Serialize;
use thiserror::Error;

/// Structured domain and infrastructure error categories exposed safely across IPC.
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Conflict detected: {0}")]
    Conflict(String),

    #[error("Authentication failed")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Safe serializable error response returned from Tauri commands.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandErrorPayload {
    pub code: String,
    pub message: String,
}

impl ApplicationError {
    /// Maps an internal application error to a structured frontend-safe error payload.
    pub fn to_command_payload(&self) -> CommandErrorPayload {
        let code = match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        };

        CommandErrorPayload {
            code: code.to_string(),
            message: self.to_string(),
        }
    }
}

impl From<rusqlite::Error> for ApplicationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error.to_string())
    }
}
