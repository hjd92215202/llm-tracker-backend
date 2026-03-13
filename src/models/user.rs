use crate::error::AppError;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::env;

// 1. 用户基础模型
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)] // 永远不要把哈希后的密码发给前端
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

// 2. 注册请求结构
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

// 3. 登录请求结构
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// 4. 认证成功响应
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

// 5. JWT 荷载结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,   // 用户 ID
    pub exp: usize, // 过期时间
}

// 6. 💡 核心：为 Claims 实现 Axum 的提取器 Trait
// 实现了这个 Trait 后，你可以在任何 Handler 的参数里直接写 `claims: Claims`
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. 获取 Authorization Header
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // 2. 验证 Bearer 格式
        if !auth_header.starts_with("Bearer ") {
            tracing::warn!("⚠️ 无效的授权头格式");
            return Err(AppError::Unauthorized);
        }

        let token = &auth_header[7..];

        // 3. 解析并验证 JWT
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_123".into());

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::error!("🚫 JWT 验证失败: {:?}", e);
            // 这里利用了 error.rs 中对 jsonwebtoken 错误的 From 实现
            AppError::Unauthorized
        })?;

        // 4. 返回 Claims
        Ok(token_data.claims)
    }
}
