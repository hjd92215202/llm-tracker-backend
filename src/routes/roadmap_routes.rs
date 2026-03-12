use crate::handlers::{note_handler, roadmap_handler};
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(roadmap_handler::list_nodes))
        .route("/", post(roadmap_handler::create_node))
        .route("/:id", put(roadmap_handler::update_node))
        .route("/:id", delete(roadmap_handler::delete_node))
        // 现在 update_node_status 的签名修复了，这里不会再报错
        .route("/:id/status", put(roadmap_handler::update_node_status))
        .route("/:id/notes", get(note_handler::get_notes_by_node))
}
