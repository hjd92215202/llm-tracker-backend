use crate::error::{AppError, AppResult};
use crate::models::roadmap::{CreateNodeRequest, RoadmapNode, UpdateNodeRequest};
use crate::repository::roadmap_repo::RoadmapRepository;
use sqlx::PgPool;

pub struct RoadmapService;

impl RoadmapService {
    /// 💡 获取用户路径树数据
    pub async fn get_roadmap_tree(pool: &PgPool, user_id: i32) -> AppResult<Vec<RoadmapNode>> {
        tracing::info!("🚀 [Roadmap Service] 开始检索用户 {} 的路径数据", user_id);
        let nodes = RoadmapRepository::fetch_all(pool, user_id).await?;
        tracing::info!("✅ [Roadmap Service] 数据加载完成, 节点数: {}", nodes.len());
        Ok(nodes)
    }

    /// 💡 添加新的学习节点
    pub async fn add_step(
        pool: &PgPool,
        req: CreateNodeRequest,
        user_id: i32,
    ) -> AppResult<RoadmapNode> {
        tracing::info!(
            "🚀 [Roadmap Service] 用户 {} 正在创建节点: '{}'",
            user_id,
            req.title
        );

        // 1. 父节点权属校验：精准检查，避免全表扫描
        if let Some(p_id) = req.parent_id {
            if !RoadmapRepository::exists(pool, p_id, user_id).await? {
                tracing::error!(
                    "🚫 [安全拦截] 用户 {} 尝试关联非本人节点 {} 作为依赖",
                    user_id,
                    p_id
                );
                return Err(AppError::AuthError("非法操作：无权关联指定的父节点".into()));
            }
        }

        let node = RoadmapRepository::create(pool, req, user_id).await?;
        tracing::info!("✅ [Roadmap Service] 节点创建成功, ID: {}", node.id);
        Ok(node)
    }

    /// 💡 快速切换学习进度
    pub async fn update_node_status(
        pool: &PgPool,
        id: i32,
        status: String,
        user_id: i32,
    ) -> AppResult<()> {
        tracing::info!(
            "🚀 [Roadmap Service] 用户 {} 更新节点 {} 状态为: {}",
            user_id,
            id,
            status
        );
        RoadmapRepository::update_status(pool, id, &status, user_id).await
    }

    /// 💡 全量配置节点
    pub async fn update_node(
        pool: &PgPool,
        id: i32,
        req: UpdateNodeRequest,
        user_id: i32,
    ) -> AppResult<()> {
        tracing::info!(
            "🚀 [Roadmap Service] 用户 {} 请求同步节点 {} 的完整配置",
            user_id,
            id
        );

        // 1. 禁止自引用逻辑
        if let Some(p_id) = req.parent_id {
            if p_id == id {
                return Err(AppError::ValidationError("节点不能将自身设为父节点".into()));
            }
            // 2. 新父节点权属校验
            if !RoadmapRepository::exists(pool, p_id, user_id).await? {
                return Err(AppError::AuthError("非法操作：目标父节点不存在".into()));
            }
        }

        RoadmapRepository::update(pool, id, req, user_id).await
    }

    /// 💡 物理删除学习节点
    pub async fn remove_node(pool: &PgPool, id: i32, user_id: i32) -> AppResult<()> {
        tracing::warn!(
            "🗑️ [Roadmap Service] 高危操作：用户 {} 物理删除节点 {}",
            user_id,
            id
        );
        RoadmapRepository::delete(pool, id, user_id).await?;
        tracing::info!("✅ [Roadmap Service] 节点 {} 已移除", id);
        Ok(())
    }
}
