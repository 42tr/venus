use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Json,
};
use crate::{
    auth::extract_uid_from_headers,
    models::{Image, ImageResponse},
};
use sqlx::SqlitePool;
use std::{fs, path::Path as StdPath};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn upload_image(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ImageResponse>, StatusCode> {
    let uid = extract_uid_from_headers(&headers)?;

    // 创建上传目录
    let upload_dir = "uploads/images";
    if !StdPath::new(upload_dir).exists() {
        fs::create_dir_all(upload_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let mut project_id: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "project_id" {
            if let Ok(data) = field.text().await {
                project_id = Some(data);
            }
            continue;
        }
        
        if name != "image" {
            continue;
        }

        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        let size = data.len() as i64;

        // 生成唯一文件名
        let id = Uuid::new_v4().to_string();
        let ext = StdPath::new(&filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin");
        let stored_filename = format!("{}.{}", id, ext);
        let file_path = format!("{}/{}", upload_dir, stored_filename);

        // 保存文件
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 获取图片尺寸（简单实现，可以使用image crate进行更精确的处理）
        let (width, height): (Option<i32>, Option<i32>) = if content_type.starts_with("image/") {
            // 这里可以使用image crate来获取真实的图片尺寸
            // 暂时返回None，后续可以优化
            (None, None)
        } else {
            (None, None)
        };

        // 保存到数据库
        let image = sqlx::query_as::<_, Image>(
            r#"
            INSERT INTO images (id, filename, original_name, mime_type, size, width, height, project_id, uploaded_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
        .bind(&id)
        .bind(&stored_filename)
        .bind(&filename)
        .bind(&content_type)
        .bind(size)
        .bind(width)
        .bind(height)
        .bind(project_id.as_deref())
        .bind(uid)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(Json(image.to_response()));
    }

    Err(StatusCode::BAD_REQUEST)
}

pub async fn get_image(
    State(pool): State<SqlitePool>,
    Path(image_id): Path<String>,
) -> Result<Response, StatusCode> {
    // 从数据库获取图片信息
    let image = sqlx::query_as::<_, Image>(
        "SELECT * FROM images WHERE id = ?"
    )
    .bind(&image_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // 读取文件
    let file_path = format!("uploads/images/{}", image.filename);
    let file_data = tokio::fs::read(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // 返回文件响应
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &image.mime_type)
        .header(header::CONTENT_LENGTH, file_data.len())
        .header(header::CACHE_CONTROL, "public, max-age=31536000") // 缓存1年
        .body(file_data.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

pub async fn list_images(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Json<Vec<ImageResponse>>, StatusCode> {
    let uid = extract_uid_from_headers(&headers)?;

    let images = sqlx::query_as::<_, Image>(
        "SELECT * FROM images WHERE uploaded_by = ? ORDER BY created_at DESC"
    )
    .bind(uid)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<ImageResponse> = images
        .into_iter()
        .map(|img| img.to_response())
        .collect();

    Ok(Json(responses))
}

pub async fn delete_image(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(image_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let uid = extract_uid_from_headers(&headers)?;

    // 获取图片信息（确保用户有权限删除）
    let image = sqlx::query_as::<_, Image>(
        "SELECT * FROM images WHERE id = ? AND uploaded_by = ?"
    )
    .bind(&image_id)
    .bind(uid)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // 先从数据库删除记录
    sqlx::query("DELETE FROM images WHERE id = ?")
        .bind(&image_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 然后删除文件
    let file_path = format!("uploads/images/{}", image.filename);
    if let Err(e) = tokio::fs::remove_file(&file_path).await {
        // 记录错误但不返回失败，因为数据库记录已经删除
        eprintln!("Warning: Failed to delete image file {}: {}", file_path, e);
    }

    Ok(StatusCode::NO_CONTENT)
}