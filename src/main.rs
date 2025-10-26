mod auth;
mod config;
mod database;
mod handlers;
mod models;

use axum::{
    extract::Request,
    http::StatusCode,
    response::{Html, Response},
    routing::{delete, get, post, put},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber;

use crate::{
    config::Config,
    database::Database,
    handlers::{
        create_project, delete_project, get_project_by_id, get_projects, update_project,
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    
    let database = Database::new(&config.database_url).await?;
    database.migrate().await?;

    let pool = database.pool();
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .route("/projects", get(get_projects).post(create_project))
        .route("/projects/:id", get(get_project_by_id).put(update_project).delete(delete_project))
        .with_state(pool.clone());

    let app = Router::new()
        .route("/", get(serve_index))
        .nest("/api", api_routes)
        // 静态文件服务 - 按照优先级顺序
        .nest_service("/assets", ServeDir::new("frontend/dist/assets"))
        .nest_service("/fonts", ServeDir::new("frontend/dist/fonts"))
        .route("/venus.svg", get(serve_static_file))
        .route("/excalidraw.css", get(serve_static_file))
        .fallback(serve_spa)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    
    tracing::info!("Server running on http://0.0.0.0:{}", config.port);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_static_file(request: Request) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    let file_path = format!("frontend/dist{}", path);
    
    match tokio::fs::read(&file_path).await {
        Ok(content) => {
            let content_type = if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".svg") {
                "image/svg+xml"
            } else {
                "application/octet-stream"
            };
            
            Ok(Response::builder()
                .header("content-type", content_type)
                .body(axum::body::Body::from(content))
                .unwrap())
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn serve_index() -> Result<Html<String>, StatusCode> {
    serve_spa().await
}

async fn serve_spa() -> Result<Html<String>, StatusCode> {
    match tokio::fs::read_to_string("frontend/dist/index.html").await {
        Ok(content) => Ok(Html(content)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}