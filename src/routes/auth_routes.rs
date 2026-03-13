use axum::{routing::{post, get}, Router};
use sqlx::PgPool;
use crate::handlers::auth_handler;

pub fn router() -> Router<PgPool> {
    Router::new()
        // 💡 公开路由：无需 Token 即可访问
        .route("/register", post(auth_handler::register))
        .route("/login", post(auth_handler::login))
        
        // 💡 私有路由：用于前端验证登录状态并获取当前用户信息
        // 访问此接口时，Axum 会自动触发 Claims 提取器逻辑进行鉴权
        .route("/me", get(auth_handler::get_me))
}