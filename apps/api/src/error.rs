use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt;

/// アプリケーション全体で使用するカスタムエラー型
#[derive(Debug)]
pub enum AppError {
    /// 内部サーバーエラー
    InternalServerError(String),
    /// バリデーションエラー
    ValidationError(String),
    /// 認証エラー
    Unauthorized(String),
    /// リソースが見つからない
    NotFound(String),
    /// リクエストが不正
    BadRequest(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InternalServerError(msg) => write!(f, "Internal server error: {msg}"),
            Self::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::BadRequest(msg) => write!(f, "Bad request: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

/// エラーレスポンスのJSON構造
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl AppError {
    /// エラーのHTTPステータスコード、エラータイプ、メッセージを取得
    #[allow(clippy::cognitive_complexity)]
    fn error_info(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::InternalServerError(_msg) => {
                // 内部エラーはログに記録するが、詳細はクライアントに返さない
                tracing::error!(error = %self, "Internal server error occurred");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error",
                    "An internal server error occurred".to_string(),
                )
            }
            Self::ValidationError(msg) => {
                tracing::warn!(error = %self, "Validation error");
                (StatusCode::BAD_REQUEST, "validation_error", msg.clone())
            }
            Self::Unauthorized(msg) => {
                tracing::warn!(error = %self, "Unauthorized access attempt");
                (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone())
            }
            Self::NotFound(msg) => {
                tracing::debug!(error = %self, "Resource not found");
                (StatusCode::NOT_FOUND, "not_found", msg.clone())
            }
            Self::BadRequest(msg) => {
                tracing::warn!(error = %self, "Bad request");
                (StatusCode::BAD_REQUEST, "bad_request", msg.clone())
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = self.error_info();

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

/// Result型のエイリアス（アプリケーション全体で使用）
pub type Result<T> = std::result::Result<T, AppError>;

/// 一般的なエラーから`AppError`への変換を実装
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        tracing::error!(error = %err, "IO error occurred");
        Self::InternalServerError(format!("IO error: {err}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!(error = %err, "JSON error occurred");
        Self::BadRequest(format!("Invalid JSON: {err}"))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "Database error occurred");
        Self::InternalServerError("Database error".to_string())
    }
}
