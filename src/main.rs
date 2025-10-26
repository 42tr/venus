mod auth;
mod auth_handlers;
mod database;
mod handlers;
mod image_handlers;
mod models;

use axum::{
    extract::Request,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

use crate::{
    auth_handlers::{get_current_user, login, register},
    database::Database,
    handlers::{create_project, delete_project, get_project_by_id, get_projects, update_project},
    image_handlers::{upload_image, get_image, list_images, delete_image},
};

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct Assets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let database = Database::new("sqlite:./venus.db").await?;
    database.migrate().await?;

    let pool = database.pool();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/user", get(get_current_user))
        .with_state(pool.clone());

    let api_routes = Router::new()
        .route("/projects", get(get_projects).post(create_project))
        .route(
            "/projects/:id",
            get(get_project_by_id)
                .put(update_project)
                .delete(delete_project),
        )
        .route("/images", post(upload_image).get(list_images))
        .route("/images/:id", get(get_image).delete(delete_image))
        .with_state(pool.clone());

    let app = Router::new()
        .route("/", get(serve_index))
        .nest("/api/auth", auth_routes)
        .nest("/api", api_routes)
        .fallback(serve_static_handler)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8085").await?;

    tracing::info!("Server running on http://0.0.0.0:8085");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_static_handler(request: Request) -> Response {
    let path = request.uri().path().trim_start_matches('/');

    // 如果路径为空，服务 index.html
    if path.is_empty() {
        return serve_index_embedded().await;
    }

    // 尝试从嵌入的文件中获取静态文件
    if let Some(content) = Assets::get(path) {
        let content_type = get_content_type(path);
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .body(axum::body::Body::from(content.data))
            .unwrap();
    }

    // 如果没找到文件，且不是 API 路径，则服务 index.html (SPA 回退)
    if !path.starts_with("api/") {
        return serve_index_embedded().await;
    }

    // 否则返回 404
    StatusCode::NOT_FOUND.into_response()
}

async fn serve_index() -> Response {
    serve_index_embedded().await
}

async fn serve_index_embedded() -> Response {
    if let Some(content) = Assets::get("index.html") {
        Html(String::from_utf8_lossy(&content.data).to_string()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

fn get_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "application/octet-stream"
    }
}
