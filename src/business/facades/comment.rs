use crate::business::mappers::generic::ToMappedList;
use crate::business::models::comment::{CommentCreateModel, CommentModel};
use crate::business::Result;
use crate::persistence::repositories::comment::CommentRepoTrait;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait CommentFacadeTrait {
    async fn list_comments_to_video(
        &self,
        video_id: i32,
        offset: Option<i64>,
    ) -> Result<Vec<CommentModel>>;
    async fn create_comment_to_video(&self, comment_model: CommentCreateModel) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct CommentFacade {
    comment_repository: Arc<dyn CommentRepoTrait + Send + Sync>,
}

impl CommentFacade {
    pub fn new(comment_repository: Arc<dyn CommentRepoTrait + Send + Sync>) -> Self {
        Self { comment_repository }
    }
}

#[async_trait]
impl CommentFacadeTrait for CommentFacade {
    async fn list_comments_to_video(
        &self,
        video_id: i32,
        offset: Option<i64>,
    ) -> Result<Vec<CommentModel>> {
        let comments_rows = self
            .comment_repository
            .fetch_comments_to_video(video_id, offset)
            .await?;

        let comments_serialized = comments_rows.to_mapped_list(CommentModel::from);

        Ok(comments_serialized)
    }

    async fn create_comment_to_video(&self, comment_model: CommentCreateModel) -> Result<()> {
        let _new_comment = self
            .comment_repository
            .create_comment_to_video(comment_model)
            .await?;

        Ok(())
    }
}
