use sqlx::PgPool;
use crate::models::artifact::{Artifact, CreateArtifactRequest};
use crate::error::AppResult;

pub struct ArtifactRepository;

impl ArtifactRepository {
    pub async fn create(pool: &PgPool, req: CreateArtifactRequest) -> AppResult<Artifact> {
        tracing::debug!("💾 SQL: 正在为笔记 {} 插入附件: {}", req.note_id, req.artifact_type);
        let artifact = sqlx::query_as::<_, Artifact>(
            r#"
            INSERT INTO artifacts (note_id, artifact_type, title, content_url)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(req.note_id)
        .bind(req.artifact_type)
        .bind(req.title)
        .bind(req.content_url)
        .fetch_one(pool)
        .await?;
        Ok(artifact)
    }

    pub async fn find_by_note(pool: &PgPool, note_id: i32) -> AppResult<Vec<Artifact>> {
        tracing::debug!("💾 SQL: 查询笔记 {} 的所有附件", note_id);
        let list = sqlx::query_as::<_, Artifact>("SELECT * FROM artifacts WHERE note_id = $1")
            .bind(note_id)
            .fetch_all(pool)
            .await?;
        Ok(list)
    }
}