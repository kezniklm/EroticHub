use crate::business::models::comment::CommentCreateModel;
use crate::persistence::entities::comment::CommentEntity;
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait CommentRepoTrait: Debug {
    async fn fetch_comments_to_video(
        &self,
        video_id: i32,
        offset: Option<i64>,
    ) -> Result<Vec<CommentEntity>>;
    async fn create_comment_to_video(&self, comment: CommentCreateModel) -> Result<CommentEntity>;
    // async fn user_comments(&self, video_id: i32) -> anyhow::Result<Vec<CommentEntity>>;
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
    async fn fetch_comments_to_video(
        &self,
        video_id: i32,
        offset: Option<i64>,
    ) -> Result<Vec<CommentEntity>> {
        let offs = offset.unwrap_or(0);
        let comments = sqlx::query_as!(
            CommentEntity,
            r#"
            SELECT id, user_id, video_id, created_at, content
            FROM comment
            WHERE video_id = $1
            ORDER BY created_at DESC
            LIMIT 10
            OFFSET $2
            "#,
            video_id,
            offs
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(comments)
    }

    async fn create_comment_to_video(&self, comment: CommentCreateModel) -> Result<CommentEntity> {
        let new_comment = sqlx::query_as!(
            CommentEntity,
            "INSERT INTO comment (video_id, user_id, content)
             VALUES ($1, $2, $3)
             RETURNING id, video_id, user_id, content, created_at",
            comment.video_id,
            comment.user_id,
            comment.content
        )
        .fetch_one(&self.pg_pool)
        .await?;

        Ok(new_comment)
    }

    // async fn user_comments(&self, user_id: i32) -> anyhow::Result<Vec<CommentEntity>> {
    //     let comments = sqlx::query_as!(
    //         CommentEntity,
    //         "SELECT * FROM comment WHERE user_id = $1",
    //         user_id
    //     )
    //     .fetch_all(&self.pg_pool)
    //     .await?;
    //
    //     Ok(comments)
    // }
}
