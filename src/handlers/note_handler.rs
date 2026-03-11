use axum::{extract::{State, Path}, Json};
use sqlx::PgPool;
use serde_json::{json, Value};
use crate::services::note_service::NoteService;
use crate::models::note::CreateNoteRequest;
use crate::models::artifact::CreateArtifactRequest;
use crate::error::AppResult;

pub async fn create_note(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateNoteRequest>,
) -> AppResult<Json<Value>> {
    tracing::info!("📬 请求创建笔记: {}", payload.title);
    let note = NoteService::create_full_note(&pool, payload).await?;
    Ok(Json(json!({ "success": true, "data": note })))
}

pub async fn get_note_detail(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<Value>> {
    tracing::info!("📬 请求笔记详情, ID: {}", id);
    let (note, artifacts) = NoteService::get_note_with_artifacts(&pool, id).await?;
    Ok(Json(json!({
        "success": true,
        "data": { "note": note, "artifacts": artifacts }
    })))
}

pub async fn get_notes_by_node(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<Value>> {
    tracing::info!("📬 请求获取节点 {} 的所有笔记", id);
    let notes = crate::repository::note_repo::NoteRepository::find_by_node(&pool, id).await?;
    Ok(Json(json!({ "success": true, "data": notes })))
}

pub async fn add_artifact(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateArtifactRequest>,
) -> AppResult<Json<Value>> {
    tracing::info!("📬 请求为笔记 {} 添加附件", payload.note_id);
    let artifact = NoteService::add_artifact_to_note(&pool, payload).await?;
    Ok(Json(json!({ "success": true, "data": artifact })))
}