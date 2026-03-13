use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use tracing::{error, warn};

/// 💡 全局 Result 别名，简化全站函数签名
pub type AppResult<T> = Result<T, AppError>;

/// 💡 全局错误枚举
/// 每一个变体都代表了一种特定的业务场景
#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error), // 500: 数据库底层故障
    NotFound(String),           // 404: 资源缺失
    Internal(String),           // 500: 未捕获的内部逻辑异常
    AuthError(String),          // 401: 身份验证失败（密码错等）
    Unauthorized,               // 401: 令牌无效或未携带
    Conflict(String),           // 409: 数据冲突（唯一索引冲突）
    ValidationError(String),    // 400: 客户端输入参数校验失败
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 1. 确定 HTTP 状态码与对外的错误消息
        let (status, error_message) = match self {
            AppError::DatabaseError(ref e) => {
                // 🔐 安全审计：记录原始 SQL 错误到日志，但不发给前端
                error!("❌ [Database Critical]: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "数据服务暂时不可用，后台已记录".to_string(),
                )
            }
            AppError::NotFound(ref msg) => {
                warn!("🔍 [Resource Missing]: {}", msg);
                (StatusCode::NOT_FOUND, msg.clone())
            }
            AppError::Internal(ref msg) => {
                error!("🔥 [Logic Error]: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::AuthError(ref msg) => {
                warn!("🔑 [Auth Failed]: {}", msg);
                (StatusCode::UNAUTHORIZED, msg.clone())
            }
            AppError::Unauthorized => {
                warn!("🚫 [Unauthorized Attempt]: Missing or corrupted JWT");
                (
                    StatusCode::UNAUTHORIZED,
                    "会话已过期或未授权，请重新登录".to_string(),
                )
            }
            AppError::Conflict(ref msg) => {
                warn!("⚠️  [Resource Conflict]: {}", msg);
                (StatusCode::CONFLICT, msg.clone())
            }
            AppError::ValidationError(ref msg) => {
                warn!("❌ [Validation Failed]: {}", msg);
                (StatusCode::BAD_REQUEST, msg.clone())
            }
        };

        // 2. 构造标准化的 JSON 响应体
        // 💡 显式类型标注 Value，适配 Rust 2024 标准
        let body: Value = json!({
            "success": false,
            "error": error_message,
        });

        (status, Json(body)).into_response()
    }
}

// --- 💡 以下是 From Trait 的自动化实现，支持 `?` 操作符 ---

// 1. 自动转换 sqlx 数据库错误
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

// 2. 自动转换底层字符串为内部错误
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

// 3. 自动转换 bcrypt 加密模块错误
impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        error!("🚨 [Bcrypt Module Failure]: {:?}", err);
        AppError::Internal("加密验证服务异常".into())
    }
}

// 4. 自动转换 jsonwebtoken 鉴权模块错误
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            // 💡 如果 Token 过期，返回更具体的提示
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AppError::AuthError("身份令牌已过期，请重新登录".to_string())
            }
            // 其余情况统一视为未授权
            _ => AppError::Unauthorized,
        }
    }
}
