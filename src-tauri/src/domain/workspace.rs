use serde::{Deserialize, Serialize};

/// Top-level local workspace or organization boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Validated input required to create a new workspace.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceInput {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
}

/// Validated input required to update an existing workspace.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceInput {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}
