use sqlx::PgPool;
use crate::models::note::{Note, CreateNoteRequest};
use crate::error::AppResult;

pub struct NoteRepository;

impl NoteRepository {
    pub async fn create(pool: &PgPool, req: CreateNoteRequest) -> AppResult<Note> {
        tracing::debug!("💾 SQL: 正在创建笔记: {}", req.title);
        let note = sqlx::query_as::<_, Note>(
            r#"
            INSERT INTO notes (node_id, title, content, tags)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(req.node_id)
        .bind(req.title)
        .bind(req.content)
        .bind(req.tags)
        .fetch_one(pool)
        .await?;
        Ok(note)
    }

    pub async fn find_by_node(pool: &PgPool, node_id: i32) -> AppResult<Vec<Note>> {
        tracing::debug!("💾 SQL: 查询节点 {} 下的所有笔记", node_id);
        let notes = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE node_id = $1 ORDER BY created_at DESC"
        )
        .bind(node_id)
        .fetch_all(pool)
        .await?;
        Ok(notes)
    }
}