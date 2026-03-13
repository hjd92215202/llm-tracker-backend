use crate::error::{AppError, AppResult};
use crate::models::artifact::{Artifact, CreateArtifactRequest};
use sqlx::PgPool;

pub struct ArtifactRepository;

impl ArtifactRepository {
    /// 💡 持久化附件数据
    /// 强制要求 user_id，确保每一条附件记录从创建伊始就属于特定用户
    pub async fn create(
        pool: &PgPool,
        req: CreateArtifactRequest,
        user_id: i32,
    ) -> AppResult<Artifact> {
        tracing::debug!(
            "💾 [SQL] 准备插入附件数据: 用户={}, 关联笔记={}, 类型={}, 标题={:?}",
            user_id,
            req.note_id,
            req.artifact_type,
            req.title
        );

        let artifact = sqlx::query_as::<_, Artifact>(
            r#"
            INSERT INTO artifacts (note_id, user_id, artifact_type, title, content_url) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING id, note_id, user_id, artifact_type, title, content_url, created_at
            "#,
        )
        .bind(req.note_id)
        .bind(user_id)
        .bind(&req.artifact_type)
        .bind(&req.title)
        .bind(&req.content_url)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "❌ [SQL Error] 插入附件记录失败 (User: {}): {:?}",
                user_id,
                e
            );
            e
        })?;

        tracing::debug!("✅ [SQL] 附件持久化完成, 生成 ID: {}", artifact.id);
        Ok(artifact)
    }

    /// 💡 检索特定笔记下的附件列表
    /// 增加 user_id 过滤，防止用户 A 看到用户 B 的私密附件
    pub async fn find_by_note(
        pool: &PgPool,
        note_id: i32,
        user_id: i32,
    ) -> AppResult<Vec<Artifact>> {
        tracing::debug!(
            "💾 [SQL] 正在检索笔记附件列表: note_id={}, user_id={}",
            note_id,
            user_id
        );

        let list = sqlx::query_as::<_, Artifact>(
            r#"
            SELECT id, note_id, user_id, artifact_type, title, content_url, created_at 
            FROM artifacts 
            WHERE note_id = $1 AND user_id = $2 
            ORDER BY created_at ASC
            "#,
        )
        .bind(note_id)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        tracing::debug!(
            "✅ [SQL] 检索完成，用户 {} 在该笔记下共有 {} 个附件",
            user_id,
            list.len()
        );
        Ok(list)
    }

    /// 💡 精确获取单个附件 (权属验证核心)
    /// 只有 ID 和 user_id 完全匹配才会返回，是 Service 层执行删除/修改前的唯一凭证
    pub async fn find_by_id(pool: &PgPool, id: i32, user_id: i32) -> AppResult<Option<Artifact>> {
        tracing::debug!(
            "💾 [SQL] 正在执行附件权属深度核验: id={}, user_id={}",
            id,
            user_id
        );

        let artifact = sqlx::query_as::<_, Artifact>(
            r#"
            SELECT id, note_id, user_id, artifact_type, title, content_url, created_at 
            FROM artifacts 
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if artifact.is_some() {
            tracing::debug!(
                "✅ [SQL] 权属核验通过: 附件 {} 确实属于用户 {}",
                id,
                user_id
            );
        } else {
            tracing::warn!(
                "🔍 [SQL] 权属核验未命中: 附件 {} 不存在或不属于用户 {}",
                id,
                user_id
            );
        }

        Ok(artifact)
    }

    /// 💡 物理删除附件
    /// 严格遵守 user_id 隔离，禁止任何越权删除行为
    pub async fn delete(pool: &PgPool, id: i32, user_id: i32) -> AppResult<()> {
        tracing::warn!(
            "💾 [SQL] 正在执行附件物理删除指令: id={}, user_id={}",
            id,
            user_id
        );

        let result = sqlx::query("DELETE FROM artifacts WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            tracing::error!(
                "❌ [SQL Error] 删除失败：尝试删除不存在或无权的附件 (ID: {})",
                id
            );
            return Err(AppError::NotFound(format!(
                "附件 {} 不存在或您无权删除",
                id
            )));
        }

        tracing::info!("✅ [SQL] 附件 {} 已从磁盘数据库记录中永久抹除", id);
        Ok(())
    }
}
