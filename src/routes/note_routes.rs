use crate::handlers::note_handler;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(note_handler::list_all_notes))
        // --- 笔记本体管理 ---
        .route("/", post(note_handler::create_note))
        .route("/:id", get(note_handler::get_note_detail))
        .route("/:id", put(note_handler::update_note))
        .route("/:id", delete(note_handler::delete_note))
        // --- 附件成果管理 ---
        .route("/artifacts", post(note_handler::add_artifact))
        // 💡 补全附件删除接口
        .route("/artifacts/:id", delete(note_handler::delete_artifact))
}
