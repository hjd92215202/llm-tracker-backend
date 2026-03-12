use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::{error, warn};

// 统一定义 Result 别名，供全系统使用
pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),   // 数据库错误
    NotFound(String),             // 404 资源不存在
    Internal(String),             // 500 内部服务器错误
    AuthError(String),            // 401 登录失败（账号或密码错误）
    Unauthorized,                 // 401 未授权（Token 无效或缺失）
    Conflict(String),             // 409 数据冲突（如用户名已存在）
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(ref e) => {
                error!("❌ [Database Error]: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "数据服务暂时不可用，请稍后再试".to_string(),
                )
            }
            AppError::NotFound(ref msg) => {
                warn!("🔍 [Not Found]: {}", msg);
                (StatusCode::NOT_FOUND, msg.clone())
            }
            AppError::Internal(ref msg) => {
                error!("🔥 [Internal Error]: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::AuthError(ref msg) => {
                warn!("🔑 [Auth Failed]: {}", msg);
                (StatusCode::UNAUTHORIZED, msg.clone())
            }
            AppError::Unauthorized => {
                warn!("🚫 [Unauthorized Access]: Missing or invalid token");
                (
                    StatusCode::UNAUTHORIZED,
                    "请先登录以访问此资源".to_string(),
                )
            }
            AppError::Conflict(ref msg) => {
                warn!("⚠️ [Conflict]: {}", msg);
                (StatusCode::CONFLICT, msg.clone())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// --- 以下是 Traits 实现，用于支持使用 `?` 自动转换错误 ---

// 1. 自动转换 sqlx 错误
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

// 2. 自动转换 String/&str 为 Internal 错误
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

// 3. 自动转换 bcrypt 加密/校验错误
impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::Internal(format!("密码加密/校验失败: {}", err))
    }
}

// 4. 自动转换 jsonwebtoken 错误
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AppError::AuthError("登录已过期，请重新登录".to_string())
            }
            _ => AppError::Unauthorized,
        }
    }
}