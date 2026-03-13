use crate::error::{AppError, AppResult};
use crate::models::artifact::{Artifact, CreateArtifactRequest};
use crate::models::note::{CreateNoteRequest, Note, UpdateNoteRequest};
use crate::repository::artifact_repo::ArtifactRepository;
use crate::repository::note_repo::NoteRepository;
use crate::repository::roadmap_repo::RoadmapRepository; 
use sqlx::PgPool;

pub struct NoteService;

impl NoteService {
    /// 💡 创建完整笔记
    /// 包含：基础参数校验、路线图节点归属权精准校验、原子化存储
    pub async fn create_full_note(
        pool: &PgPool,
        req: CreateNoteRequest,
        user_id: i32,
    ) -> AppResult<Note> {
        tracing::info!("🚀 [Note Service] 用户 {} 发起笔记创建请求: title='{}'", user_id, req.title);

        // 1. 基础字段合法性校验
        if req.title.trim().is_empty() || req.content.trim().is_empty() {
            tracing::warn!("⚠️ [Note Service] 用户 {} 提交了空的标题或内容", user_id);
            return Err(AppError::ValidationError("笔记标题和内容不能为空".into()));
        }

        // 2. 节点归属权精准校验：确保笔记关联的路线节点属于当前用户
        if let Some(n_id) = req.node_id {
            tracing::debug!("🔍 [Note Service] 正在核验路线节点归属: node_id={}, user_id={}", n_id, user_id);
            let node_exists = RoadmapRepository::exists(pool, n_id, user_id).await?;
            if !node_exists {
                tracing::error!("🚫 [安全拦截] 用户 {} 尝试将笔记关联至非法节点 {}", user_id, n_id);
                return Err(AppError::AuthError("非法操作：目标学习节点不存在或无权访问".into()));
            }
        }

        // 3. 执行持久化
        let note = NoteRepository::create(pool, req, user_id)
            .await
            .map_err(|e| {
                tracing::error!("🔥 [Note Service] 数据库持久化异常: {:?}", e);
                e
            })?;

        tracing::info!("✅ [Note Service] 笔记创建成功, ID: {}, Title: '{}'", note.id, note.title);
        Ok(note)
    }

    /// 💡 获取笔记详情及其所有成果附件
    /// 业务逻辑：执行双重归属校验，确保用户 A 无法通过 ID 获取用户 B 的私有笔记或附件
    pub async fn get_note_with_artifacts(
        pool: &PgPool,
        note_id: i32,
        user_id: i32,
    ) -> AppResult<(Note, Vec<Artifact>)> {
        tracing::info!("📖 [Note Service] 用户 {} 正在读取笔记详情, ID: {}", user_id, note_id);

        // 1. 获取并核验笔记归属权
        let note = NoteRepository::find_by_id(pool, note_id, user_id)
            .await?
            .ok_or_else(|| {
                tracing::warn!("🔍 [Note Service] 检索未命中或无权访问: ID {}", note_id);
                AppError::NotFound(format!("找不到指定的笔记(ID:{})", note_id))
            })?;

        // 2. 获取该笔记关联的附件
        let artifacts = ArtifactRepository::find_by_note(pool, note_id, user_id).await?;

        tracing::info!("✅ [Note Service] 笔记 {} 数据加载完成, 关联附件数: {}", note_id, artifacts.len());
        Ok((note, artifacts))
    }

    /// 💡 获取用户的所有笔记列表 (用于后台管理页)
    pub async fn get_all_user_notes(pool: &PgPool, user_id: i32) -> AppResult<Vec<Note>> {
        tracing::info!("🚀 [Note Service] 正在为用户 {} 检索全量笔记库", user_id);
        
        // 💡 这里的 NoteRepository 需补全对应 SQL: SELECT * FROM notes WHERE user_id = $1
        let notes = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        
        tracing::info!("✅ [Note Service] 全量检索完成，笔记总计: {}", notes.len());
        Ok(notes)
    }

    /// 💡 获取特定路线图节点下的笔记列表
    pub async fn get_node_notes(pool: &PgPool, node_id: i32, user_id: i32) -> AppResult<Vec<Note>> {
        tracing::debug!("📁 [Note Service] 用户 {} 检索节点笔记列表: node_id={}", user_id, node_id);

        // 安全前置校验：确保节点本身属于该用户
        if !RoadmapRepository::exists(pool, node_id, user_id).await? {
            tracing::error!("🚫 [安全拦截] 用户 {} 尝试拉取非法节点的笔记列表: {}", user_id, node_id);
            return Err(AppError::AuthError("无权访问该节点的资源".into()));
        }

        let notes = NoteRepository::find_by_node(pool, node_id, user_id).await?;
        tracing::debug!("✅ [Note Service] 节点 {} 笔记列表加载完成, Count: {}", node_id, notes.len());
        Ok(notes)
    }

    /// 💡 更新笔记内容
    pub async fn update_note(
        pool: &PgPool,
        id: i32,
        req: UpdateNoteRequest,
        user_id: i32,
    ) -> AppResult<()> {
        tracing::info!("📝 [Note Service] 用户 {} 正在执行笔记内容更新, ID: {}", user_id, id);

        if req.title.trim().is_empty() {
            return Err(AppError::ValidationError("更新标题不能为空".into()));
        }

        NoteRepository::update(pool, id, req, user_id).await?;

        tracing::info!("✅ [Note Service] 笔记 {} 状态已同步", id);
        Ok(())
    }

    /// 💡 物理删除笔记 (级联删除附件)
    pub async fn remove_note(pool: &PgPool, id: i32, user_id: i32) -> AppResult<()> {
        tracing::warn!("🗑️ [Note Service] 高危操作: 用户 {} 发起笔记物理删除, ID: {}", user_id, id);

        NoteRepository::delete(pool, id, user_id).await?;

        tracing::info!("✅ [Note Service] 笔记 {} 及其关联数据已永久移除", id);
        Ok(())
    }

    /// 💡 为笔记添加学习成果附件
    pub async fn add_artifact_to_note(
        pool: &PgPool,
        req: CreateArtifactRequest,
        user_id: i32,
    ) -> AppResult<Artifact> {
        tracing::info!("📎 [Note Service] 用户 {} 尝试为笔记 {} 注册成果附件", user_id, req.note_id);

        // 1. 链接非空校验
        if req.content_url.trim().is_empty() {
            return Err(AppError::ValidationError("成果链接(Content URL)不能为空".into()));
        }

        // 2. 权属预审：确保目标笔记属于该用户
        let _host_check = NoteRepository::find_by_id(pool, req.note_id, user_id)
            .await?
            .ok_or_else(|| {
                tracing::error!("🚫 [安全拦截] 用户 {} 尝试非法修改他人笔记附件, ID: {}", user_id, req.note_id);
                AppError::AuthError("操作失败：目标笔记不存在或权限不足".into())
            })?;

        // 3. 执行持久化
        let artifact = ArtifactRepository::create(pool, req, user_id).await?;

        tracing::info!("✅ [Note Service] 附件已成功关联至笔记 {}, Artifact ID: {}", artifact.note_id, artifact.id);
        Ok(artifact)
    }

    /// 💡 移除特定成果附件
    pub async fn remove_artifact(pool: &PgPool, artifact_id: i32, user_id: i32) -> AppResult<()> {
        tracing::info!("🗑️ [Note Service] 用户 {} 请求删除成果附件, ID: {}", user_id, artifact_id);

        // 1. 前置校验：只有确定附件属于该用户时才允许删除
        // 此逻辑激活了 ArtifactRepository::find_by_id 链路
        let _ = ArtifactRepository::find_by_id(pool, artifact_id, user_id)
            .await?
            .ok_or_else(|| {
                tracing::warn!("🚫 [越权尝试] 用户 {} 尝试删除不存在或无权的附件: {}", user_id, artifact_id);
                AppError::NotFound("附件不存在或无权操作".into())
            })?;

        // 2. 执行逻辑物理删除
        ArtifactRepository::delete(pool, artifact_id, user_id).await?;

        tracing::info!("✨ [Note Service] 附件(ID:{}) 移除指令已成功执行", artifact_id);
        Ok(())
    }
}