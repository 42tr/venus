use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::{
    auth::{generate_jwt_token, hash_password, verify_password},
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse},
};

pub async fn register(
    State(pool): State<SqlitePool>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // 检查用户名是否已存在
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ? OR email = ?"
    )
    .bind(&req.username)
    .bind(&req.email)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT); // 409 Conflict - 用户已存在
    }

    // 哈希密码
    let password_hash = hash_password(&req.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let now = Utc::now();

    // 创建新用户
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id, username, email, password_hash, created_at, updated_at
        "#
    )
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 生成JWT token
    let token = generate_jwt_token(user.id, user.username.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = AuthResponse {
        user: UserResponse::from(user),
        token,
    };

    Ok(Json(response))
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // 查找用户
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    // 验证密码
    let password_valid = verify_password(&req.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !password_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 生成JWT token
    let token = generate_jwt_token(user.id, user.username.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = AuthResponse {
        user: UserResponse::from(user),
        token,
    };

    Ok(Json(response))
}

pub async fn get_current_user(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, StatusCode> {
    let uid = crate::auth::extract_uid_from_headers(&headers)?;
    
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE id = ?"
    )
    .bind(uid)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user.ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse::from(user)))
}