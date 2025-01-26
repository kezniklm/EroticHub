use crate::business::mappers::generic::ToMappedList;
use crate::business::models::video_category::VideoCategory;
use crate::persistence::repositories::video_category::VideoCategoryRepoTrait;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait VideoCategoryFacadeTrait {
    async fn list_categories(&self) -> anyhow::Result<Vec<VideoCategory>>;
}

#[derive(Debug, Clone)]
pub struct VideoCategoryFacade {
    category_repository: Arc<dyn VideoCategoryRepoTrait + Send + Sync>,
}

impl VideoCategoryFacade {
    pub fn new(category_repository: Arc<dyn VideoCategoryRepoTrait + Send + Sync>) -> Self {
        Self {
            category_repository,
        }
    }
}

#[async_trait]
impl VideoCategoryFacadeTrait for VideoCategoryFacade {
    async fn list_categories(&self) -> anyhow::Result<Vec<VideoCategory>> {
        let categories_rows = self.category_repository.fetch_categories().await?;

        let categories = categories_rows.to_mapped_list(VideoCategory::from);

        Ok(categories)
    }
}
