use axum::{routing::{get, post}, Router};
use sqlx::PgPool;
use crate::handlers::note_handler;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", post(note_handler::create_note))
        .route("/:id", get(note_handler::get_note_detail))
        .route("/artifacts", post(note_handler::add_artifact))
}