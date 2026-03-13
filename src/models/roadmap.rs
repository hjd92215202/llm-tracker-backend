use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 💡 学习路径节点模型
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RoadmapNode {
    pub id: i32,
    pub user_id: i32,           // 💡 新增：用于多用户隔离，标志该节点属于哪个用户
    pub title: String,
    pub description: Option<String>,
    pub status: String,         // todo, in_progress, completed
    pub node_type: String,       // theory, coding, project
    pub parent_id: Option<i32>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 💡 创建节点的请求结构
/// user_id 将由后端从 Claims 中提取，不通过前端参数传递，防止身份伪造
#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub title: String,
    pub description: Option<String>,
    pub node_type: Option<String>,
    pub parent_id: Option<i32>,
    pub sort_order: Option<i32>,
}

/// 💡 更新节点的请求结构
#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub node_type: String,
    pub parent_id: Option<i32>,
    pub sort_order: i32,
}