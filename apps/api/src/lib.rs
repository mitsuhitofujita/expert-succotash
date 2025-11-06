pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod store;

use axum::{
    Json, Router,
    routing::{delete, get, post, put},
};
pub use db::init_db_pool;
use error::Result;
pub use repository::{AttendanceEventRepository, UserRepository};
use serde::Serialize;
pub use store::TodoStore;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}

async fn health_check() -> Result<Json<HealthResponse>> {
    tracing::info!("Health check endpoint called");
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
    }))
}

/// Error handling test endpoint
/// Only available in debug builds or test environments
#[cfg(any(debug_assertions, test))]
async fn test_error_internal() -> Result<Json<HealthResponse>> {
    Err(error::AppError::InternalServerError(
        "This is a test internal error".to_string(),
    ))
}

#[cfg(any(debug_assertions, test))]
async fn test_error_validation() -> Result<Json<HealthResponse>> {
    Err(error::AppError::ValidationError(
        "Invalid input provided".to_string(),
    ))
}

#[cfg(any(debug_assertions, test))]
async fn test_error_unauthorized() -> Result<Json<HealthResponse>> {
    Err(error::AppError::Unauthorized(
        "Invalid credentials".to_string(),
    ))
}

#[cfg(any(debug_assertions, test))]
async fn test_error_notfound() -> Result<Json<HealthResponse>> {
    Err(error::AppError::NotFound("Resource not found".to_string()))
}

#[cfg(any(debug_assertions, test))]
async fn test_error_badrequest() -> Result<Json<HealthResponse>> {
    Err(error::AppError::BadRequest(
        "Invalid request format".to_string(),
    ))
}

/// Create the application router
/// This function is public to allow testing
pub fn create_router(store: TodoStore) -> Router {
    // Router configuration
    #[cfg_attr(not(any(debug_assertions, test)), allow(unused_mut))]
    let mut app = Router::new()
        .route("/health", get(health_check))
        // Todo CRUD endpoints
        .route("/api/todos", get(handlers::get_todos))
        .route("/api/todos", post(handlers::create_todo))
        .route("/api/todos/{id}", get(handlers::get_todo))
        .route("/api/todos/{id}", put(handlers::update_todo))
        .route("/api/todos/{id}", delete(handlers::delete_todo))
        .with_state(store);

    // Error handling test endpoints (only available in debug builds or test environments)
    #[cfg(any(debug_assertions, test))]
    {
        tracing::warn!("Test error endpoints are enabled (debug/test mode only)");
        app = app
            .route("/test/error/internal", get(test_error_internal))
            .route("/test/error/validation", get(test_error_validation))
            .route("/test/error/unauthorized", get(test_error_unauthorized))
            .route("/test/error/notfound", get(test_error_notfound))
            .route("/test/error/badrequest", get(test_error_badrequest));
    }

    // Add HTTP request/response tracing
    app.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    )
}
