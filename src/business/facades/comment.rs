use crate::business::mappers::generic::ToMappedList;
use crate::business::models::comment::CommentModel;
use crate::persistence::repositories::comment::CommentRepoTrait;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait CommentFacadeTrait {
    async fn list_comments_to_video(&self, video_id: i32) -> anyhow::Result<Vec<CommentModel>>;
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
    async fn list_comments_to_video(&self, video_id: i32) -> anyhow::Result<Vec<CommentModel>> {
        let comments_rows = self
            .comment_repository
            .list_comments_to_video(video_id)
            .await?;

        let comments_serialized = comments_rows.to_mapped_list(CommentModel::from);

        Ok(comments_serialized)
    }
}
