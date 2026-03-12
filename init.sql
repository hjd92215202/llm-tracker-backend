-- 1. 学习路径节点表 (Roadmap Nodes)
CREATE TABLE roadmap_nodes (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'todo',      -- todo, in_progress, completed
    node_type VARCHAR(50) DEFAULT 'theory', -- theory, coding, project
    parent_id INTEGER REFERENCES roadmap_nodes(id) ON DELETE CASCADE,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. 笔记表 (Notes)
-- 存储实际的文字内容
CREATE TABLE notes (
    id SERIAL PRIMARY KEY,
    node_id INTEGER REFERENCES roadmap_nodes(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,                   -- 存储 Markdown 全文
    summary TEXT,                            -- AI 生成的简要总结（用于搜索预览）
    tags TEXT[],
    is_indexed BOOLEAN DEFAULT FALSE,        -- 标记是否已同步到 Qdrant
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 3. 学习成果/附件表 (Learning Artifacts)
-- 专门记录你的实验结果，如训练日志路径、Demo链接、生成的图片等
CREATE TABLE artifacts (
    id SERIAL PRIMARY KEY,
    note_id INTEGER REFERENCES notes(id) ON DELETE CASCADE,
    artifact_type VARCHAR(50),               -- code_snippet, model_weight, demo_url, image
    title VARCHAR(255),
    content_url TEXT,                        -- 外部链接或文件路径
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 触发器更新 updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_roadmap_modtime BEFORE UPDATE ON roadmap_nodes FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
CREATE TRIGGER update_notes_modtime BEFORE UPDATE ON notes FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- 1. 用户表
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. 修改现有表，增加 user_id 关联 (个人系统可先不强制，但建议加上)
ALTER TABLE roadmap_nodes ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE notes ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE;