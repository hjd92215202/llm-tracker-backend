use sqlx::PgPool;
use crate::models::user::{User, RegisterRequest};
use crate::error::AppResult;

pub struct UserRepository;

impl UserRepository {
    pub async fn create(pool: &PgPool, req: RegisterRequest, hash: String) -> AppResult<User> {
        tracing::debug!("💾 SQL: 正在注册新用户: {}", req.username);
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(req.username)
        .bind(req.email)
        .bind(hash)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    pub async fn find_by_username(pool: &PgPool, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await?;
        Ok(user)
    }
}