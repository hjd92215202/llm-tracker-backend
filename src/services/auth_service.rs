use crate::models::user::{User, RegisterRequest, LoginRequest, AuthResponse, Claims};
use crate::repository::user_repo::UserRepository;
use crate::error::{AppError, AppResult};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use sqlx::PgPool;
use std::env;
use chrono::{Utc, Duration};

pub struct AuthService;

impl AuthService {
    /// 处理用户注册
    pub async fn register(pool: &PgPool, req: RegisterRequest) -> AppResult<User> {
        tracing::info!("🚀 [Auth Service] 正在尝试注册新用户: {}", req.username);

        // 1. 唯一性检查：检查用户名是否已被占用
        if UserRepository::find_by_username(pool, &req.username).await?.is_some() {
            tracing::warn!("⚠️ 注册失败: 用户名 {} 已存在", req.username);
            return Err(AppError::Conflict("该用户名已被占用".into()));
        }

        // 2. 密码哈希处理
        // 💡 修复点：使用 &req.password 获取引用，不移动 req 的所有权
        let password_hash = hash(&req.password, DEFAULT_COST)?;
        tracing::debug!("🔐 用户 {} 的密码哈希计算完成", req.username);

        // 3. 调用仓库层保存数据
        // 此时 req 依然是完整的，可以被移动到 create 函数中
        let new_user = UserRepository::create(pool, req, password_hash).await?;

        tracing::info!("✅ 用户 {} 注册成功，ID: {}", new_user.username, new_user.id);
        Ok(new_user)
    }

    /// 处理用户登录并签发 Token
    pub async fn login(pool: &PgPool, req: LoginRequest) -> AppResult<AuthResponse> {
        tracing::info!("🔑 [Auth Service] 正在验证用户登录: {}", req.username);

        // 1. 获取用户信息
        let user = UserRepository::find_by_username(pool, &req.username).await?
            .ok_or_else(|| {
                tracing::warn!("🔍 登录失败: 用户 {} 不存在", req.username);
                AppError::AuthError("账号或密码不正确".into())
            })?;

        // 2. 验证密码有效性
        // 💡 修复点：使用 &req.password
        let is_valid = verify(&req.password, &user.password_hash)?;
        if !is_valid {
            tracing::warn!("❌ 登录失败: 用户 {} 密码错误", req.username);
            return Err(AppError::AuthError("账号或密码不正确".into()));
        }

        // 3. 生成 JWT Token
        tracing::debug!("🛰️ 正在为用户 {} 准备签发访问令牌", user.username);
        
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::error!("🚨 环境变量缺失: JWT_SECRET 未设置！使用不安全的默认值。");
            "dangerous_default_secret_please_change_immediately".into()
        });

        // 定义 24 小时后过期
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("非法的时间戳计算")
            .timestamp() as usize;

        let claims = Claims {
            sub: user.id,
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )?;

        tracing::info!("✨ 用户 {} 认证成功，令牌已签发", user.username);
        
        Ok(AuthResponse {
            token,
            user,
        })
    }
}