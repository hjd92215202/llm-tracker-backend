use sqlx::PgPool;
use crate::repository::roadmap_repo::RoadmapRepository;
use crate::models::roadmap::{RoadmapNode, CreateNodeRequest};
use crate::error::AppResult;

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
}