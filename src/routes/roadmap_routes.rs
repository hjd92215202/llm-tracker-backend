use crate::handlers::{note_handler, roadmap_handler};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

pub fn router() -> Router<PgPool> {
    Router::new()
        // 💡 学习路径节点核心操作
        // 获取当前登录用户的所有路径节点 (已实现 user_id 物理隔离)
        .route("/", get(roadmap_handler::list_nodes))
        // 在当前用户名下创建新的学习节点
        .route("/", post(roadmap_handler::create_node))
        
        // 💡 特定节点管理 (RESTful 风格)
        // 更新节点的完整配置（标题、描述、依赖关系、排序等）
        .route("/:id", put(roadmap_handler::update_node))
        // 永久删除节点及其级联的所有笔记与附件
        .route("/:id", delete(roadmap_handler::delete_node))

        // 💡 状态快调路由
        // 专门用于快速切换学习状态 (todo -> in_progress -> completed)
        .route("/:id/status", put(roadmap_handler::update_node_status))

        // 💡 关联查询路由
        // 获取指定路径节点下属于当前用户的所有学习笔记
        .route("/:id/notes", get(note_handler::get_notes_by_node))
}