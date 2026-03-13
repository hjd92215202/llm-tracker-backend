use crate::error::AppResult;
use crate::models::user::{Claims, LoginRequest, RegisterRequest};
use crate::services::auth_service::AuthService;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use sqlx::PgPool;

/// 💡 [POST] 用户注册
/// 协议层：接收注册载荷，转发至 AuthService 执行哈希与存储
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<Value>> {
    tracing::info!(
        "📬 [Auth Handler] 收到新用户注册请求: username='{}', email='{}'",
        payload.username,
        payload.email
    );

    let user = AuthService::register(&pool, payload).await?;

    // 💡 显式标注类型以适配 Rust 2024 兼容性
    let response: Value = json!({
        "success": true,
        "data": user
    });

    tracing::info!("✅ [Auth Handler] 用户注册逻辑处理完成");
    Ok(Json(response))
}

/// 💡 [POST] 用户登录
/// 协议层：核验凭证并签发 JWT
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<Value>> {
    tracing::info!(
        "🔑 [Auth Handler] 用户 '{}' 正在尝试登录系统",
        payload.username
    );

    let auth_data = AuthService::login(&pool, payload).await?;

    let response: Value = json!({
        "success": true,
        "data": auth_data
    });

    tracing::info!(
        "✨ [Auth Handler] 用户 '{}' 身份核验通过，已下发令牌",
        auth_data.user.username
    );
    Ok(Json(response))
}

/// 💡 [GET] 获取当前登录用户信息
/// 协议层：利用 Claims 提取器自动鉴权。
/// 如果 Token 无效，Axum 会在进入此函数前直接返回 401 错误。
pub async fn get_me(claims: Claims, State(pool): State<PgPool>) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!("📡 [Auth Handler] 正在为用户 {} 同步本地登录状态", user_id);

    // 💡 优化：此处不再直接编写 SQL，而是通过 UserRepository 检索
    // 确保不查询 password_hash 字段
    let user = crate::repository::user_repo::UserRepository::find_by_id(&pool, user_id)
        .await?
        .ok_or_else(|| {
            tracing::error!("🔥 [数据异常] 令牌有效但数据库未找到用户: {}", user_id);
            crate::error::AppError::NotFound("用户账户已失效或不存在".into())
        })?;

    let response: Value = json!({
        "success": true,
        "data": user
    });

    tracing::debug!("✅ [Auth Handler] 用户 {} 数据同步成功", user.username);
    Ok(Json(response))
}
