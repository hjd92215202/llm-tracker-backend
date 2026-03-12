use axum::{routing::post, Router};
use sqlx::PgPool;
use crate::handlers::auth_handler;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/register", post(auth_handler::register))
        .route("/login", post(auth_handler::login))
}