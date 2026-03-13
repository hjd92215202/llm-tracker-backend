use crate::error::{AppError, AppResult};
use crate::models::roadmap::{CreateNodeRequest, RoadmapNode, UpdateNodeRequest};
use sqlx::PgPool;

pub struct RoadmapRepository;

impl RoadmapRepository {
    /// 💡 获取属于当前用户的所有路径节点
    pub async fn fetch_all(pool: &PgPool, user_id: i32) -> AppResult<Vec<RoadmapNode>> {
        tracing::debug!("💾 [SQL] 检索用户 {} 的学习路径全图", user_id);
        let nodes = sqlx::query_as::<_, RoadmapNode>(
            r#"
            SELECT id, user_id, title, description, status, node_type, parent_id, sort_order, created_at, updated_at 
            FROM roadmap_nodes 
            WHERE user_id = $1 
            ORDER BY sort_order ASC, id ASC
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(nodes)
    }

    /// 💡 检查特定节点是否属于该用户 (用于 Service 层校验)
    pub async fn exists(pool: &PgPool, id: i32, user_id: i32) -> AppResult<bool> {
        let result = sqlx::query("SELECT 1 FROM roadmap_nodes WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
        Ok(result.is_some())
    }

    /// 💡 创建学习路径节点
    pub async fn create(
        pool: &PgPool,
        req: CreateNodeRequest,
        user_id: i32,
    ) -> AppResult<RoadmapNode> {
        tracing::debug!("💾 [SQL] 用户 {} 正在创建新节点: '{}'", user_id, req.title);
        let node = sqlx::query_as::<_, RoadmapNode>(
            r#"
            INSERT INTO roadmap_nodes (title, description, node_type, parent_id, sort_order, user_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, title, description, status, node_type, parent_id, sort_order, created_at, updated_at
            "#
        )
        .bind(req.title)
        .bind(req.description)
        .bind(req.node_type.unwrap_or_else(|| "theory".to_string()))
        .bind(req.parent_id)
        .bind(req.sort_order.unwrap_or(0))
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(node)
    }

    /// 💡 快速更新节点状态
    pub async fn update_status(
        pool: &PgPool,
        id: i32,
        status: &str,
        user_id: i32,
    ) -> AppResult<()> {
        let result = sqlx::query("UPDATE roadmap_nodes SET status = $1, updated_at = NOW() WHERE id = $2 AND user_id = $3")
            .bind(status)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("目标节点不存在或您无权修改".into()));
        }
        Ok(())
    }

    /// 💡 完整更新节点配置
    pub async fn update(
        pool: &PgPool,
        id: i32,
        req: UpdateNodeRequest,
        user_id: i32,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE roadmap_nodes 
            SET title = $1, description = $2, status = $3, node_type = $4, parent_id = $5, sort_order = $6, updated_at = NOW()
            WHERE id = $7 AND user_id = $8
            "#
        )
        .bind(&req.title).bind(&req.description).bind(&req.status).bind(&req.node_type)
        .bind(req.parent_id).bind(req.sort_order).bind(id).bind(user_id)
        .execute(pool).await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("操作失败：节点不存在或权限不足".into()));
        }
        Ok(())
    }

    /// 💡 物理删除节点
    pub async fn delete(pool: &PgPool, id: i32, user_id: i32) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM roadmap_nodes WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("删除失败：指定的节点不存在".into()));
        }
        Ok(())
    }
}
