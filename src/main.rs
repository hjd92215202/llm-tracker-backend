use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer}; // 增加了 Any
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod error;
mod models;
mod repository;
mod services;
mod handlers;
mod routes;

#[tokio::main]
async fn main() {
    // 1. 第一时间加载环境变量
    dotenvy::dotenv().ok();

    // 2. 初始化日志系统 (确保在所有逻辑之前)
    // 注意：如果你的包名是 llm-tracker-backend，过滤规则必须写成 llm_tracker_backend
    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_target(false) 
            .with_ansi(true) // 开启彩色输出
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "llm_tracker_backend=info,tower_http=debug,sqlx=warn".into()
        }))
        .init();

    tracing::info!("🚀 大模型学习记录系统后端正在启动...");

    // 3. 数据库连接处理
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::error!("❌ 错误: 未在环境变量或 .env 中找到 DATABASE_URL");
        std::process::exit(1);
    });

    tracing::info!("📡 正在尝试连接数据库...");

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await 
    {
        Ok(p) => {
            tracing::info!("✅ 数据库连接成功！");
            p
        },
        Err(e) => {
            tracing::error!("❌ 数据库连接失败: {}", e);
            tracing::error!("💡 请确保 Postgres 已启动且 .env 配置正确");
            std::process::exit(1);
        }
    };

    // 4. 跨域与中间件配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 5. 路由组装
    let app = Router::new()
        .nest("/api/roadmap", routes::roadmap_routes::router())
        .nest("/api/notes", routes::note_routes::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http()) // 此层负责打印每个 HTTP 请求的详细日志
        .with_state(pool);

    // 6. 服务绑定
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => {
            tracing::info!("🌐 服务运行于: http://{}", addr);
            l
        },
        Err(e) => {
            tracing::error!("❌ 无法绑定端口 {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    // 7. 启动
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("🔥 服务器运行异常: {}", e);
    }
}