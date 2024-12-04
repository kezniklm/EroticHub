use crate::persistence::entities::comment::CommentEntity;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait CommentRepoTrait: Debug {
    async fn list_comments_to_video(&self, video_id: i32) -> anyhow::Result<Vec<CommentEntity>>;
    async fn user_comments(&self, video_id: i32) -> anyhow::Result<Vec<CommentEntity>>;
}

#[derive(Debug, Clone)]
pub struct CommentRepository {
    pg_pool: PgPool,
}

impl CommentRepository {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl CommentRepoTrait for CommentRepository {
    async fn list_comments_to_video(&self, video_id: i32) -> anyhow::Result<Vec<CommentEntity>> {
        let comments = sqlx::query_as!(
            CommentEntity,
            "SELECT * FROM comment WHERE video_id = $1",
            video_id
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(comments)
    }

    async fn user_comments(&self, user_id: i32) -> anyhow::Result<Vec<CommentEntity>> {
        let comments = sqlx::query_as!(
            CommentEntity,
            "SELECT * FROM comment WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(comments)
    }
}
