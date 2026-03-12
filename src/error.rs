use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;

// 统一定义 Result 别名，供全系统使用
pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(ref e) => {
                error!("❌ [数据库严重错误]: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "数据库操作失败，请检查后端日志".to_string(),
                )
            }
            AppError::NotFound(ref msg) => {
                error!("🔍 [资源不存在]: {}", msg);
                (StatusCode::NOT_FOUND, msg.clone())
            }
            AppError::Internal(ref msg) => {
                error!("🔥 [内部逻辑错误]: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Internal(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Internal(err.to_string())
    }
}
