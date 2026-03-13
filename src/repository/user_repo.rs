use crate::error::AppResult;
use crate::models::user::{RegisterRequest, User};
use sqlx::PgPool;

pub struct UserRepository;

impl UserRepository {
    /// 💡 创建新用户
    /// 执行持久化操作，并将密码哈希与元数据安全存入数据库
    pub async fn create(pool: &PgPool, req: RegisterRequest, hash: String) -> AppResult<User> {
        tracing::debug!(
            "💾 [SQL] 准备持久化新用户: username={}, email={}",
            req.username,
            req.email
        );

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash) 
            VALUES ($1, $2, $3) 
            RETURNING id, username, email, password_hash, created_at
            "#,
        )
        .bind(&req.username)
        .bind(&req.email)
        .bind(hash) // hash 已经是 move 进来的，直接绑定
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ [SQL Error] 持久化用户 {} 失败: {:?}", req.username, e);
            e
        })?;

        tracing::info!("✅ [SQL] 用户注册成功, Assigned ID: {}", user.id);
        Ok(user)
    }

    /// 💡 根据用户名检索用户
    /// 核心用途：登录身份核验
    pub async fn find_by_username(pool: &PgPool, username: &str) -> AppResult<Option<User>> {
        tracing::debug!("💾 [SQL] 正在执行身份检索: username={}", username);

        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, created_at 
            FROM users 
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ [SQL Error] 查询用户名 {} 出错: {:?}", username, e);
            e
        })?;

        if user.is_some() {
            tracing::debug!("✅ [SQL] 身份检索成功: {}", username);
        } else {
            tracing::debug!("🔍 [SQL] 未发现匹配用户: {}", username);
        }

        Ok(user)
    }

    /// 💡 根据电子邮箱检索用户
    /// 核心用途：注册阶段唯一性核验
    pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
        tracing::debug!("💾 [SQL] 正在执行邮箱冲突检查: email={}", email);

        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, created_at 
            FROM users 
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ [SQL Error] 查询邮箱 {} 出错: {:?}", email, e);
            e
        })?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &PgPool, id: i32) -> AppResult<Option<User>> {
        tracing::debug!("💾 [SQL] 正在根据 ID 检索用户摘要: {}", id);

        let user = sqlx::query_as::<_, User>(
            r#"
        SELECT id, username, email, created_at 
        FROM users 
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }
}
