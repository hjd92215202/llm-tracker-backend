use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Artifact {
    pub id: i32,
    pub note_id: i32,
    pub artifact_type: String, // code_snippet, model_weight, demo_url, image
    pub title: Option<String>,
    pub content_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateArtifactRequest {
    pub note_id: i32,
    pub artifact_type: String,
    pub title: Option<String>,
    pub content_url: String,
}
