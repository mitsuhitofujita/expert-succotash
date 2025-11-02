mod error;

use axum::{Json, Router, routing::get};
use error::Result;
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health_check() -> Result<Json<HealthResponse>> {
    tracing::info!("Health check endpoint called");
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
    }))
}

/// エラーハンドリングのテスト用エンドポイント
/// デバッグビルドまたはテスト環境でのみ有効
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

/// tracingの初期化
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // tracingの初期化
    init_tracing();

    tracing::info!("Starting API server");

    // ルーティングの設定
    #[cfg_attr(not(any(debug_assertions, test)), allow(unused_mut))]
    let mut app = Router::new().route("/health", get(health_check));

    // エラーハンドリングのテスト用エンドポイント（デバッグビルドまたはテスト環境でのみ有効）
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

    // HTTPリクエスト/レスポンスのトレーシングを追加
    let app = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    );

    // サーバーアドレスの設定
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    // サーバーの起動
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
