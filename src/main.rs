use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, time::Duration};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod error;
mod handlers;
mod models;
mod repository;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    // --- 1. 环境初始化 ---
    // 加载 .env 文件，如果加载失败则跳过（在生产环境中环境变量通常由容器系统注入）
    if let Err(e) = dotenvy::dotenv() {
        // 只有当 .env 文件存在但解析失败时才警告
        if !e.to_string().contains("not found") {
            eprintln!("⚠️ 加载 .env 环境变量失败: {}", e);
            std::process::exit(1);
        }

        if std::env::var("JWT_SECRET").is_err() {
            tracing::error!("❌ [Fatal] 必须设置环境变量 JWT_SECRET 才能启动服务");
            std::process::exit(1);
        }
    }

    // --- 2. 结构化日志系统初始化 ---
    // 配置彩色控制台输出，并根据 RUST_LOG 环境变量过滤日志
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_ansi(true) // 开启彩色输出
                .with_thread_ids(true), // 记录线程 ID，方便并发调试
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            // 💡 默认日志策略：项目核心 info 级别，SQLx 只记录 warn 级别
            "llm_tracker_backend=info,tower_http=debug,sqlx=warn".into()
        }))
        .init();

    tracing::info!("🚀 [System] 大模型学习记录系统后端正在启动...");

    // --- 3. 数据库连接池初始化 ---
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::error!("🚨 致命错误: 未在环境变量中发现 DATABASE_URL");
        std::process::exit(1);
    });

    tracing::info!("📡 [Database] 正在初始化 PostgreSQL 连接池...");
    let pool = PgPoolOptions::new()
        .max_connections(15) // 最大并发连接数
        .min_connections(5) // 保持至少 5 个空闲连接
        .acquire_timeout(Duration::from_secs(5)) // 获取连接的超时时间
        .idle_timeout(Duration::from_secs(600)) // 空闲连接在 10 分钟后回收
        .connect(&db_url)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("❌ [Database] 无法建立数据库连接: {}", e);
            tracing::error!("💡 请确保 Postgres 容器已启动且端口 5432 已映射至本地");
            std::process::exit(1);
        });
    tracing::info!("✅ [Database] 数据库连接池就绪");

    // --- 4. 跨域策略 (CORS) 配置 ---
    // 在开发阶段允许所有来源，生产环境应按需缩小范围
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // --- 5. 组装路由体系 ---
    let app = Router::new()
        // 健康检查接口
        .route("/health", get(|| async { "🚀 System is healthy" }))
        // 业务路由模块化嵌套
        .nest("/api/roadmap", routes::roadmap_routes::router())
        .nest("/api/notes", routes::note_routes::router())
        .nest("/api/auth", routes::auth_routes::router())
        // --- 中间件栈 ---
        .layer(cors)
        .layer(TraceLayer::new_for_http()) // 💡 此处会自动记录请求路径、状态码和耗时
        .with_state(pool);

    // --- 6. 绑定服务地址 ---
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => {
            tracing::info!("🌐 [Network] API 服务已绑定至: http://{}", addr);
            l
        }
        Err(e) => {
            tracing::error!("❌ [Network] 端口绑定失败 (3000): {}", e);
            std::process::exit(1);
        }
    };

    // --- 7. 启动服务器并配置优雅停机 ---
    tracing::info!("✨ [Core] 服务器正式进入监听状态...");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|e| {
            tracing::error!("🔥 [Critical] 运行时发生不可恢复的错误: {}", e);
        });

    tracing::info!("👋 [System] 优雅停机流程完成，再见！");
}

/// 💡 实现优雅停机监听
/// 捕捉 Ctrl+C 或系统的终止信号，确保退出前清理连接
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("❌ 无法监听 Ctrl+C 信号");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("❌ 无法监听 SIGTERM 信号")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::warn!("🛑 [Signal] 接收到中断信号 (Ctrl+C)，开始执行退出前清理..."); },
        _ = terminate => { tracing::warn!("🛑 [Signal] 接收到终止信号 (SIGTERM)，开始执行退出前清理..."); },
    }
}
