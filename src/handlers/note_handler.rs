use crate::error::AppResult;
use crate::models::artifact::CreateArtifactRequest;
use crate::models::note::{CreateNoteRequest, UpdateNoteRequest};
use crate::models::user::Claims;
use crate::services::note_service::NoteService;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use sqlx::PgPool;

/// 💡 [POST] /api/notes
/// 协议层：创建笔记。强制从 Claims 提取身份，杜绝 ID 篡改。
pub async fn create_note(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateNoteRequest>,
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!(
        "📬 [Handler] 用户 {} 正在发起笔记创建: '{}'",
        user_id,
        payload.title
    );

    let note = NoteService::create_full_note(&pool, payload, user_id).await?;

    let response: Value = json!({
        "success": true,
        "data": note
    });

    tracing::info!(
        "✅ [Handler] 用户 {} 笔记创建成功, ID: {}",
        user_id,
        note.id
    );
    Ok(Json(response))
}

/// 💡 [GET] /api/notes/:id
/// 协议层：获取笔记详情及关联附件。
pub async fn get_note_detail(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!("📬 [Handler] 用户 {} 正在请求笔记详情, ID: {}", user_id, id);

    let (note, artifacts) = NoteService::get_note_with_artifacts(&pool, id, user_id).await?;

    let response: Value = json!({
        "success": true,
        "data": {
            "note": note,
            "artifacts": artifacts
        }
    });

    tracing::debug!(
        "✅ [Handler] 笔记详情(ID:{}) 数据及附件(count:{}) 已下发",
        id,
        artifacts.len()
    );
    Ok(Json(response))
}

/// 💡 [GET] /api/roadmap/:id/notes
/// 协议层：获取指定路线节点下的所有笔记。
pub async fn get_notes_by_node(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(node_id): Path<i32>,
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!(
        "📬 [Handler] 用户 {} 检索节点 {} 的笔记资源列表",
        user_id,
        node_id
    );

    let notes = NoteService::get_node_notes(&pool, node_id, user_id).await?;

    let response: Value = json!({
        "success": true,
        "data": notes
    });

    tracing::debug!(
        "✅ [Handler] 节点 {} 下的 {} 篇笔记检索成功",
        node_id,
        notes.len()
    );
    Ok(Json(response))
}

/// 💡 [PUT] /api/notes/:id
/// 协议层：更新笔记内容。
pub async fn update_note(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateNoteRequest>,
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!("📬 [Handler] 用户 {} 请求更新笔记内容, ID: {}", user_id, id);

    NoteService::update_note(&pool, id, payload, user_id).await?;

    let response: Value = json!({
        "success": true,
        "msg": "内容同步成功"
    });

    tracing::info!("✅ [Handler] 笔记 {} 已完成数据刷新", id);
    Ok(Json(response))
}

/// 💡 [DELETE] /api/notes/:id
/// 协议层：物理删除笔记及级联附件。
pub async fn delete_note(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::warn!(
        "🗑️ [Handler] 高危操作: 用户 {} 正在尝试删除笔记(ID:{})",
        user_id,
        id
    );

    NoteService::remove_note(&pool, id, user_id).await?;

    let response: Value = json!({
        "success": true,
        "msg": "笔记及关联数据已永久移除"
    });

    tracing::info!("✅ [Handler] 笔记 {} 删除动作执行完毕", id);
    Ok(Json(response))
}

/// 💡 [POST] /api/notes/artifacts
/// 协议层：为笔记挂载研究成果附件。
pub async fn add_artifact(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateArtifactRequest>,
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!(
        "📬 [Handler] 用户 {} 正在为笔记 {} 注册新附件",
        user_id,
        payload.note_id
    );

    let artifact = NoteService::add_artifact_to_note(&pool, payload, user_id).await?;

    let response: Value = json!({
        "success": true,
        "data": artifact
    });

    tracing::info!(
        "✅ [Handler] 附件(ID:{}) 已成功关联至笔记 {}",
        artifact.id,
        artifact.note_id
    );
    Ok(Json(response))
}

/// 💡 [DELETE] /api/notes/artifacts/:id
/// 协议层：单独删除某个研究成果附件。 (全量优化新增)
pub async fn delete_artifact(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::warn!("🗑️ [Handler] 用户 {} 请求删除特定附件, ID: {}", user_id, id);

    NoteService::remove_artifact(&pool, id, user_id).await?;

    let response: Value = json!({
        "success": true,
        "msg": "附件已移除"
    });

    tracing::info!("✅ [Handler] 附件 {} 移除逻辑执行完毕", id);
    Ok(Json(response))
}

pub async fn list_all_notes(claims: Claims, State(pool): State<PgPool>) -> AppResult<Json<Value>> {
    let user_id = claims.sub;
    tracing::info!("📬 用户 {} 请求获取全量笔记列表", user_id);

    // 调用仓库层 (需在 note_repo 补一个简单的 find_all 方法)
    let notes = NoteService::get_all_user_notes(&pool, user_id).await?;

    Ok(Json(json!({ "success": true, "data": notes })))
}
