use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: i32,
    pub node_id: Option<i32>,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_indexed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub node_id: Option<i32>,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}