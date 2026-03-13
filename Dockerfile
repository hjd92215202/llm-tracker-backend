# 阶段 1: 构建
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /usr/src/app
# 安装构建依赖
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# 复制工程文件
COPY . .

# 构建 release 版本
RUN cargo build --release

# 阶段 2: 运行
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# 从构建阶段复制二进制文件
COPY --from=builder /usr/src/app/target/release/llm-tracker-backend /app/backend
# 复制配置文件
COPY .env_tem .env

EXPOSE 3000
CMD ["./backend"]