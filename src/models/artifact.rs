use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 💡 学习成果/附件模型
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Artifact {
    pub id: i32,
    pub note_id: i32,
    pub user_id: i32,          // 💡 新增：用于多用户隔离，确保附件归属权
    pub artifact_type: String, // 类型枚举: code_snippet, model_weight, demo_url, image
    pub title: Option<String>,
    pub content_url: String,
    pub created_at: DateTime<Utc>,
}

/// 💡 创建附件的请求结构
/// 注意：user_id 不需要由前端通过 JSON 传入，而是由后端从 JWT Token 中提取
#[derive(Debug, Deserialize)]
pub struct CreateArtifactRequest {
    pub note_id: i32,
    pub artifact_type: String,
    pub title: Option<String>,
    pub content_url: String,
}