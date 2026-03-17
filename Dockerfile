# --- 阶段 1: 构建阶段 (Builder) ---
# 使用官方最新的 Rust 镜像进行编译
FROM rust:latest AS builder

# 设置工作目录
WORKDIR /usr/src/app

# 安装构建依赖 (OpenSSL 开发库和编译器工具)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 1. 拷贝所有源代码
COPY . .

# 2. 编译 Release 版本
# 使用 --release 优化运行速度
RUN cargo build --release

# --- 阶段 2: 运行阶段 (Runner) ---
# 使用体积更小的 Debian 稳定版作为运行底座
FROM debian:bookworm-slim

# 安装运行时的必要库 (如 SSL 证书和 OpenSSL 运行库)
# 这是访问数据库和签发 JWT 必须的
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段只拷贝编译好的二进制文件
# 注意：这里的 llm-tracker-backend 必须与你 Cargo.toml 里的 [package] name 一致
COPY --from=builder /usr/src/app/target/release/llm-tracker-backend /app/backend

# 暴露后端服务端口 (Axum 默认 3000)
EXPOSE 3000

# 运行程序
CMD ["./backend"]