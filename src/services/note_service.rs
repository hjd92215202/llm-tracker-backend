use crate::error::{AppError, AppResult};
use crate::models::artifact::{Artifact, CreateArtifactRequest};
use crate::models::note::{CreateNoteRequest, Note};
use crate::repository::artifact_repo::ArtifactRepository;
use crate::repository::note_repo::NoteRepository;
use sqlx::PgPool;

pub struct NoteService;

impl NoteService {
    pub async fn create_full_note(pool: &PgPool, req: CreateNoteRequest) -> AppResult<Note> {
        tracing::info!("🚀 业务逻辑: 开始保存学习笔记: {}", req.title);
        NoteRepository::create(pool, req).await
    }

    pub async fn get_note_with_artifacts(
        pool: &PgPool,
        note_id: i32,
    ) -> AppResult<(Note, Vec<Artifact>)> {
        tracing::info!("🚀 业务逻辑: 获取笔记详情及其附件, ID: {}", note_id);

        let note = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = $1")
            .bind(note_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("笔记 ID {} 未找到", note_id)))?;

        let artifacts = ArtifactRepository::find_by_note(pool, note_id).await?;
        Ok((note, artifacts))
    }

    pub async fn add_artifact_to_note(
        pool: &PgPool,
        req: CreateArtifactRequest,
    ) -> AppResult<Artifact> {
        tracing::info!("🚀 业务逻辑: 为笔记 {} 添加学习成果附件", req.note_id);
        ArtifactRepository::create(pool, req).await
    }
}
