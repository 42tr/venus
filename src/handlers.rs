use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use chrono::Utc;
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    auth::extract_uid_from_headers,
    models::{CreateProjectRequest, Project, ProjectRow, ProjectSummary, UpdateProjectRequest}
};

pub async fn get_projects(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProjectSummary>>, StatusCode> {
    let uid = extract_uid_from_headers(&headers)?;
    
    let project_rows = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, name, content, uid, created_at, updated_at FROM projects WHERE uid = ? ORDER BY created_at DESC"
    )
    .bind(uid)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summaries: Vec<ProjectSummary> = project_rows
        .into_iter()
        .map(|row| Project::from(row).into())
        .collect();
    Ok(Json(summaries))
}

pub async fn create_project(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    let uid = extract_uid_from_headers(&headers)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let default_content = json!({
        "elements": [],
        "appState": {"collaborators": []},
        "files": {}
    });

    let project_row = sqlx::query_as::<_, ProjectRow>(
        r#"
        INSERT INTO projects (id, name, content, uid, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id, name, content, uid, created_at, updated_at
        "#
    )
    .bind(&id)
    .bind(&req.name)
    .bind(default_content.to_string())
    .bind(uid)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Project::from(project_row)))
}

pub async fn get_project_by_id(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    let uid = extract_uid_from_headers(&headers)?;
    
    let project_row = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, name, content, uid, created_at, updated_at FROM projects WHERE id = ? AND uid = ?"
    )
    .bind(&id)
    .bind(uid)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match project_row {
        Some(row) => Ok(Json(Project::from(row))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_project(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let uid = extract_uid_from_headers(&headers)?;
    let now = Utc::now();
    
    let result = sqlx::query(
        "UPDATE projects SET content = ?, updated_at = ? WHERE id = ? AND uid = ?"
    )
    .bind(req.content.to_string())
    .bind(now)
    .bind(&id)
    .bind(uid)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(json!({"status": "success"})))
}

pub async fn delete_project(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let uid = extract_uid_from_headers(&headers)?;
    
    let result = sqlx::query("DELETE FROM projects WHERE id = ? AND uid = ?")
        .bind(&id)
        .bind(uid)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}