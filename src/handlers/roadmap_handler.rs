use crate::error::AppResult;
use crate::models::roadmap::{CreateNodeRequest, UpdateNodeRequest};
use crate::services::roadmap_service::RoadmapService;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use sqlx::PgPool;

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
    let status = payload["status"]
        .as_str()
        .ok_or_else(|| crate::error::AppError::Internal("缺少 status 字段".into()))?;

    tracing::info!("📬 请求更新节点 {} 状态为 {}", id, status);
    RoadmapService::update_node_status(&pool, id, status.to_string()).await?;

    Ok(Json(json!({ "success": true, "msg": "状态已更新" })))
}

pub async fn update_node(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateNodeRequest>,
) -> AppResult<Json<Value>> {
    tracing::info!("📬 请求进入: 更新路线图节点内容, ID: {}", id);

    RoadmapService::update_node(&pool, id, payload).await?;

    Ok(Json(json!({
        "success": true,
        "msg": format!("节点 {} 已成功更新", id)
    })))
}

pub async fn delete_node(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<Value>> {
    tracing::warn!("📬 请求进入: 删除路线图节点, ID: {}", id);

    crate::services::roadmap_service::RoadmapService::remove_node(&pool, id).await?;

    Ok(Json(json!({
        "success": true,
        "msg": format!("节点 {} 及其子节点已成功移除", id)
    })))
}
