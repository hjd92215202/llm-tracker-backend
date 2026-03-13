use crate::error::{AppError, AppResult};
use crate::models::user::{AuthResponse, Claims, LoginRequest, RegisterRequest, User};
use crate::repository::user_repo::UserRepository;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use std::env;

pub struct AuthService;

impl AuthService {
    /// 💡 处理用户注册
    /// 解决了 "cannot move out of req because it is borrowed" 的冲突
    pub async fn register(pool: &PgPool, req: RegisterRequest) -> AppResult<User> {
        tracing::info!("🚀 [Auth Service] 收到注册请求，用户名: {}", req.username);

        // --- 1. 参数合法性校验 (直接调用，不产生长期借用变量) ---
        if req.username.trim().is_empty() {
            return Err(AppError::ValidationError("用户名不能为空".into()));
        }
        if req.email.trim().is_empty() || !req.email.contains('@') {
            return Err(AppError::ValidationError("邮箱格式非法".into()));
        }
        if req.password.len() < 6 {
            return Err(AppError::ValidationError("密码至少需要6位".into()));
        }

        // --- 2. 唯一性核验 ---
        // 校验用户名是否重复
        if UserRepository::find_by_username(pool, req.username.trim())
            .await?
            .is_some()
        {
            tracing::warn!("⚠️ [注册拦截] 用户名 '{}' 已被占用", req.username);
            return Err(AppError::Conflict("该用户名已被占用".into()));
        }

        // 校验邮箱是否重复
        if UserRepository::find_by_email(pool, req.email.trim())
            .await?
            .is_some()
        {
            tracing::warn!("⚠️ [注册拦截] 邮箱 '{}' 已被注册", req.email);
            return Err(AppError::Conflict("该邮箱已被注册".into()));
        }

        // --- 3. 密码安全哈希 ---
        // 这里只是临时借用 &req.password，这行结束后借用即解除
        tracing::debug!("🔐 正在为用户 {} 计算 Bcrypt 哈希...", req.username);
        let password_hash = hash(&req.password, DEFAULT_COST)?;

        // --- 4. 存储数据 (执行所有权转移 Move) ---
        // 💡 重点：由于上面没有任何变量还在“借用”req，这里可以安全 move
        let user_name_for_log = req.username.clone(); // 提前克隆一份用于打印日志
        let user = UserRepository::create(pool, req, password_hash)
            .await
            .map_err(|e| {
                tracing::error!(
                    "🔥 [Database Error] 写入用户 {} 失败: {:?}",
                    user_name_for_log,
                    e
                );
                e
            })?;

        tracing::info!(
            "✅ [注册成功] 用户 {} 已入库, ID: {}",
            user.username,
            user.id
        );
        Ok(user)
    }

    /// 💡 处理用户登录
    pub async fn login(pool: &PgPool, req: LoginRequest) -> AppResult<AuthResponse> {
        tracing::info!("🔑 [Auth Service] 尝试认证用户: {}", req.username);

        // 1. 查找用户
        let user = UserRepository::find_by_username(pool, req.username.trim())
            .await?
            .ok_or_else(|| {
                tracing::warn!("🔍 [认证失败] 用户不存在: {}", req.username);
                AppError::AuthError("账号或密码不正确".into())
            })?;

        // 2. 密码核验 (使用引用借用)
        let is_valid = verify(&req.password, &user.password_hash)?;
        if !is_valid {
            tracing::warn!("❌ [认证失败] 用户 {} 密码错误", req.username);
            return Err(AppError::AuthError("账号或密码不正确".into()));
        }

        // 3. 生成身份令牌
        tracing::debug!("📡 正在为 {} 签发 JWT...", user.username);
        let secret = env::var("JWT_SECRET").map_err(|_| {
            tracing::error!("🚨 环境变量 JWT_SECRET 缺失，拒绝签发 Token");
            AppError::Internal("服务器安全配置缺失".into())
        })?;

        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .ok_or_else(|| AppError::Internal("时间计算溢出".into()))?;

        let claims = Claims {
            sub: user.id,
            exp: expiration.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| {
            tracing::error!("🔥 JWT 签发失败: {:?}", e);
            AppError::Internal("令牌生成异常".into())
        })?;

        tracing::info!("✨ [登录成功] 用户 {} 已获得访问令牌", user.username);
        Ok(AuthResponse { token, user })
    }
}
