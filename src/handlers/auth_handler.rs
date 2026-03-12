use axum::{extract::State, Json};
use sqlx::PgPool;
use serde_json::{json, Value};
use crate::services::auth_service::AuthService;
use crate::models::user::{RegisterRequest, LoginRequest};
use crate::error::AppResult;

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<Value>> {
    let user = AuthService::register(&pool, payload).await?;
    Ok(Json(json!({ "success": true, "data": user })))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<Value>> {
    let auth_data = AuthService::login(&pool, payload).await?;
    Ok(Json(json!({ "success": true, "data": auth_data })))
}