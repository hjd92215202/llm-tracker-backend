use crate::error::AppResult;
use crate::models::roadmap::{CreateNodeRequest, RoadmapNode, UpdateNodeRequest};
use crate::repository::roadmap_repo::RoadmapRepository;
use sqlx::PgPool;

pub struct RoadmapService;

impl RoadmapService {
    pub async fn get_roadmap_tree(pool: &PgPool) -> AppResult<Vec<RoadmapNode>> {
        tracing::info!("🚀 业务逻辑: 正在获取完整学习路径树");
        RoadmapRepository::fetch_all(pool).await
    }

    pub async fn add_step(pool: &PgPool, req: CreateNodeRequest) -> AppResult<RoadmapNode> {
        tracing::info!("🚀 业务逻辑: 正在添加学习步骤: {}", req.title);
        RoadmapRepository::create(pool, req).await
    }

    pub async fn update_node_status(pool: &PgPool, id: i32, status: String) -> AppResult<()> {
        tracing::info!("🚀 业务逻辑: 更新节点 {} 的学习状态为 {}", id, status);
        RoadmapRepository::update_status(pool, id, &status).await
    }

    pub async fn update_node(pool: &PgPool, id: i32, req: UpdateNodeRequest) -> AppResult<()> {
        tracing::info!("🚀 业务逻辑: 正在处理节点 {} 的完整信息更新请求", id);

        // 可以在此处增加校验逻辑：例如 parent_id 不能等于自己的 id，防止死循环
        if let Some(p_id) = req.parent_id {
            if p_id == id {
                return Err(crate::error::AppError::Internal(
                    "节点的父节点不能指向自己".into(),
                ));
            }
        }

        RoadmapRepository::update(pool, id, req).await
    }

    pub async fn remove_node(pool: &PgPool, id: i32) -> AppResult<()> {
        tracing::warn!("🚀 业务逻辑: 触发节点物理删除, ID: {}", id);
        RoadmapRepository::delete(pool, id).await
    }
}
