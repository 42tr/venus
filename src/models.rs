use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, FromRow)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub content: String, // JSON string from database
    pub uid: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub content: serde_json::Value, // Parsed JSON object
    pub uid: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self {
        let content = serde_json::from_str(&row.content)
            .unwrap_or(serde_json::json!({})); // Fallback to empty object if parsing fails
        
        Self {
            id: row.id,
            name: row.name,
            content,
            uid: row.uid,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub content: serde_json::Value,
}

impl From<Project> for ProjectSummary {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
        }
    }
}