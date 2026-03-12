use crate::error::AppResult;
use crate::models::roadmap::{CreateNodeRequest, RoadmapNode, UpdateNodeRequest};
use sqlx::PgPool;

pub struct RoadmapRepository;

impl RoadmapRepository {
    pub async fn fetch_all(pool: &PgPool) -> AppResult<Vec<RoadmapNode>> {
        tracing::debug!("💾 SQL: 正在查询所有路线图节点");
        let nodes = sqlx::query_as::<_, RoadmapNode>(
            "SELECT * FROM roadmap_nodes ORDER BY sort_order ASC, id ASC",
        )
        .fetch_all(pool)
        .await?;
        Ok(nodes)
    }

    pub async fn create(pool: &PgPool, req: CreateNodeRequest) -> AppResult<RoadmapNode> {
        tracing::debug!("💾 SQL: 正在插入新节点: {}", req.title);
        let node = sqlx::query_as::<_, RoadmapNode>(
            r#"
            INSERT INTO roadmap_nodes (title, description, node_type, parent_id, sort_order)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(req.title)
        .bind(req.description)
        .bind(req.node_type.unwrap_or_else(|| "theory".to_string()))
        .bind(req.parent_id)
        .bind(req.sort_order.unwrap_or(0))
        .fetch_one(pool)
        .await?;
        Ok(node)
    }

    pub async fn update_status(pool: &PgPool, id: i32, status: &str) -> AppResult<()> {
        tracing::debug!("💾 SQL: 正在更新节点 {} 状态为 {}", id, status);
        sqlx::query("UPDATE roadmap_nodes SET status = $1 WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(pool: &PgPool, id: i32, req: UpdateNodeRequest) -> AppResult<()> {
        tracing::debug!("💾 SQL: 执行更新节点动作, ID: {}", id);

        let result = sqlx::query(
            r#"
            UPDATE roadmap_nodes 
            SET title = $1, description = $2, status = $3, node_type = $4, parent_id = $5, sort_order = $6, updated_at = NOW()
            WHERE id = $7
            "#
        )
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.status)
        .bind(&req.node_type)
        .bind(req.parent_id)
        .bind(req.sort_order)
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound(format!(
                "找不到 ID 为 {} 的节点进行更新",
                id
            )));
        }

        Ok(())
    }
}
