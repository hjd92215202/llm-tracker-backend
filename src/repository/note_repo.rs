use crate::error::{AppError, AppResult};
use crate::models::note::{CreateNoteRequest, Note, UpdateNoteRequest};
use sqlx::PgPool;

pub struct NoteRepository;

impl NoteRepository {
    /// 💡 创建笔记
    /// 将用户提交的内容持久化，并强制绑定 user_id 建立归属权
    pub async fn create(pool: &PgPool, req: CreateNoteRequest, user_id: i32) -> AppResult<Note> {
        tracing::debug!(
            "💾 [SQL] 正在为用户 {} 创建新笔记: title='{}', node_id={:?}",
            user_id,
            req.title,
            req.node_id
        );

        let note = sqlx::query_as::<_, Note>(
            r#"
            INSERT INTO notes (node_id, title, content, tags, user_id) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING id, node_id, user_id, title, content, summary, tags, is_indexed, created_at, updated_at
            "#,
        )
        .bind(req.node_id)
        .bind(&req.title)
        .bind(&req.content)
        .bind(&req.tags)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ [SQL Error] 用户 {} 笔记插入失败: {:?}", user_id, e);
            e
        })?;

        tracing::info!("✅ [SQL] 笔记创建成功, ID: {}, 用户: {}", note.id, user_id);
        Ok(note)
    }

    /// 💡 获取指定节点下的所有笔记
    /// 严格通过 user_id 过滤，返回当前用户在该知识节点的成果
    pub async fn find_by_node(pool: &PgPool, node_id: i32, user_id: i32) -> AppResult<Vec<Note>> {
        tracing::debug!(
            "💾 [SQL] 正在检索用户 {} 的节点笔记列表: node_id={}",
            user_id,
            node_id
        );

        let notes = sqlx::query_as::<_, Note>(
            r#"
            SELECT id, node_id, user_id, title, content, summary, tags, is_indexed, created_at, updated_at 
            FROM notes 
            WHERE node_id = $1 AND user_id = $2 
            ORDER BY created_at DESC
            "#
        )
        .bind(node_id)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        tracing::debug!("✅ [SQL] 检索完成，找到 {} 篇笔记", notes.len());
        Ok(notes)
    }

    /// 💡 获取单篇笔记详情
    /// 强制增加 user_id 校验，防止用户通过 URL 猜测 ID 访问他人笔记
    pub async fn find_by_id(pool: &PgPool, id: i32, user_id: i32) -> AppResult<Option<Note>> {
        tracing::debug!("💾 [SQL] 正在验证笔记详情: id={}, user_id={}", id, user_id);

        let note = sqlx::query_as::<_, Note>(
            r#"
            SELECT id, node_id, user_id, title, content, summary, tags, is_indexed, created_at, updated_at 
            FROM notes 
            WHERE id = $1 AND user_id = $2
            "#
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(note)
    }

    /// 💡 更新笔记内容
    /// 仅在 id 与 user_id 同时匹配时执行更新
    pub async fn update(
        pool: &PgPool,
        id: i32,
        req: UpdateNoteRequest,
        user_id: i32,
    ) -> AppResult<()> {
        tracing::debug!("💾 [SQL] 准备更新笔记: id={}, 用户={}", id, user_id);

        let result = sqlx::query(
            r#"
            UPDATE notes 
            SET title = $1, content = $2, tags = $3, updated_at = NOW() 
            WHERE id = $4 AND user_id = $5
            "#,
        )
        .bind(&req.title)
        .bind(&req.content)
        .bind(&req.tags)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            tracing::warn!(
                "⚠️ [SQL] 更新失败: 笔记 {} 不存在或不属于用户 {}",
                id,
                user_id
            );
            return Err(AppError::NotFound(format!(
                "笔记(ID:{})不存在或无权访问",
                id
            )));
        }

        tracing::info!("✅ [SQL] 笔记 {} 更新成功", id);
        Ok(())
    }

    /// 💡 删除笔记
    /// 严格隔离删除，物理移除用户数据
    pub async fn delete(pool: &PgPool, id: i32, user_id: i32) -> AppResult<()> {
        tracing::warn!("💾 [SQL] 正在删除用户 {} 的笔记: ID={}", user_id, id);

        let result = sqlx::query("DELETE FROM notes WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            tracing::warn!("⚠️ [SQL] 删除操作未命中: ID {} 用户 {}", id, user_id);
            return Err(AppError::NotFound(format!(
                "未找到 ID 为 {} 的笔记进行删除",
                id
            )));
        }

        tracing::info!("✅ [SQL] 笔记 {} 已从数据库中移除", id);
        Ok(())
    }
}
