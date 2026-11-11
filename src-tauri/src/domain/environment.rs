use serde::{Deserialize, Serialize};

use crate::domain::errors::ApplicationError;

/// Canonical runtime environment categories scoped under a project.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EnvironmentKind {
    Local,
    Development,
    Testing,
    Staging,
}

impl EnvironmentKind {
    /// Ordered list of default environments created automatically with every new project.
    pub const STANDARD_DEFAULTS: [EnvironmentKind; 4] =
        [Self::Local, Self::Development, Self::Testing, Self::Staging];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Development => "development",
            Self::Testing => "testing",
            Self::Staging => "staging",
        }
    }

    pub fn default_display_name(self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Development => "Development",
            Self::Testing => "Testing",
            Self::Staging => "Staging",
        }
    }

    pub fn parse(raw_kind: &str) -> Result<Self, ApplicationError> {
        match raw_kind.trim().to_ascii_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "development" => Ok(Self::Development),
            "testing" => Ok(Self::Testing),
            "staging" => Ok(Self::Staging),
            other => Err(ApplicationError::Validation(format!(
                "Unsupported environment kind: {other}"
            ))),
        }
    }
}

/// Isolated environment belonging to a single project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub slug: String,
    pub kind: EnvironmentKind,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Input to create a custom environment inside a project.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnvironmentInput {
    pub project_id: String,
    pub name: String,
    pub slug: Option<String>,
    pub kind: EnvironmentKind,
    pub is_default: bool,
}
