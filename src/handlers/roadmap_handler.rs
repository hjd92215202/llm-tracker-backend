use crate::error::AppResult;
use crate::models::roadmap::{CreateNodeRequest, UpdateNodeRequest};
use crate::models::user::Claims;
use crate::services::roadmap_service::RoadmapService;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use sqlx::PgPool;

/// 💡 [GET] 获取当前用户的完整路线图
/// 协议层：通过鉴权提取器强制校验身份，并检索该用户私有的路径节点
pub async fn list_nodes(
    claims: Claims, 
    State(pool): State<PgPool>
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!("📬 [Roadmap Handler] 用户 {} 请求获取学习路径全表", user_id);

    let nodes = RoadmapService::get_roadmap_tree(&pool, user_id).await?;
    
    let response: Value = json!({ 
        "success": true, 
        "data": nodes 
    });

    tracing::debug!("✅ [Roadmap Handler] 用户 {} 的 {} 个路径节点已打包下发", user_id, nodes.len());
    Ok(Json(response))
}

/// 💡 [POST] 创建新的路线图节点
/// 协议层：接收节点配置，透传 user_id 确保数据物理隔离
pub async fn create_node(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateNodeRequest>, // Json 必须在最后
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!("📬 [Roadmap Handler] 用户 {} 正在创建新节点: '{}'", user_id, payload.title);

    let node = RoadmapService::add_step(&pool, payload, user_id).await?;
    
    let response: Value = json!({ 
        "success": true, 
        "data": node 
    });

    tracing::info!("✅ [Roadmap Handler] 新节点创建成功, ID: {}", node.id);
    Ok(Json(response))
}

/// 💡 [PUT] 快速更新节点学习状态
/// 协议层：解析简易载荷，触发状态变更逻辑
pub async fn update_node_status(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<Value>, // Json 必须在最后
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    
    // 从动态 JSON 中安全提取状态字符串
    let status = payload["status"]
        .as_str()
        .ok_or_else(|| {
            tracing::warn!("⚠️ [Roadmap Handler] 用户 {} 提交的更新请求缺少 status 字段", user_id);
            crate::error::AppError::Internal("缺少 status 字段".into())
        })?;

    tracing::info!("📬 [Roadmap Handler] 用户 {} 尝试更新节点 {} 状态为: {}", user_id, id, status);
    
    RoadmapService::update_node_status(&pool, id, status.to_string(), user_id).await?;

    let response: Value = json!({ 
        "success": true, 
        "msg": "学习进度已同步" 
    });

    tracing::info!("✅ [Roadmap Handler] 节点 {} 状态变更已生效", id);
    Ok(Json(response))
}

/// 💡 [PUT] 更新节点完整详细配置
/// 协议层：处理包括标题、描述、父子依赖关系在内的全量更新
pub async fn update_node(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateNodeRequest>, // Json 必须在最后
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!("📬 [Roadmap Handler] 用户 {} 正在更新节点 {} 的全量配置", user_id, id);

    RoadmapService::update_node(&pool, id, payload, user_id).await?;

    let response: Value = json!({
        "success": true,
        "msg": format!("节点(ID:{})配置更新成功", id)
    });

    tracing::info!("✅ [Roadmap Handler] 节点 {} 数据同步完成", id);
    Ok(Json(response))
}

/// 💡 [DELETE] 永久移除路径节点
/// 协议层：触发级联删除逻辑
pub async fn delete_node(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::warn!("🗑️ [Roadmap Handler] 高危操作: 用户 {} 正在请求删除路线图节点, ID: {}", user_id, id);

    RoadmapService::remove_node(&pool, id, user_id).await?;

    let response: Value = json!({
        "success": true,
        "msg": "学习节点及其关联内容已成功移除"
    });

    tracing::info!("✅ [Roadmap Handler] 节点 {} 删除指令执行完毕", id);
    Ok(Json(response))
}