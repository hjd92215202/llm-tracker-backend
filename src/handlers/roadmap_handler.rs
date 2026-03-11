use axum::{extract::{State, Path}, Json};
use sqlx::PgPool;
use serde_json::{json, Value};
use crate::services::roadmap_service::RoadmapService;
use crate::models::roadmap::CreateNodeRequest;
use crate::error::AppResult;

pub async fn list_nodes(State(pool): State<PgPool>) -> AppResult<Json<Value>> {
    tracing::info!("📬 请求获取路线图列表");
    let nodes = RoadmapService::get_roadmap_tree(&pool).await?;
    Ok(Json(json!({ "success": true, "data": nodes })))
}

pub async fn create_node(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateNodeRequest>,
) -> AppResult<Json<Value>> {
    tracing::info!("📬 请求创建路线图节点: {}", payload.title);
    let node = RoadmapService::add_step(&pool, payload).await?;
    Ok(Json(json!({ "success": true, "data": node })))
}

// 修复点：参数必须写成 Json<Value>，且返回类型必须包含 Json<> 包裹
pub async fn update_node_status(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<Value>, 
) -> AppResult<Json<Value>> {
    let status = payload["status"].as_str()
        .ok_or_else(|| crate::error::AppError::Internal("缺少 status 字段".into()))?;
    
    tracing::info!("📬 请求更新节点 {} 状态为 {}", id, status);
    RoadmapService::update_node_status(&pool, id, status.to_string()).await?;
    
    Ok(Json(json!({ "success": true, "msg": "状态已更新" })))
}