use serde::{Deserialize, Serialize};

/// Application or service tracked inside a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Validated input required to create a project and initialize its default environments.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub workspace_id: String,
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
}

/// Validated input required to update project metadata.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectInput {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}
