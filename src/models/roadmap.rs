use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RoadmapNode {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub node_type: String,
    pub parent_id: Option<i32>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub title: String,
    pub description: Option<String>,
    pub node_type: Option<String>,
    pub parent_id: Option<i32>,
    pub sort_order: Option<i32>,
}