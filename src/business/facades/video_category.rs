use crate::business::mappers::generic::ToMappedList;
use crate::business::models::video_category::{VideoCategory, VideoCategorySelected};
use crate::business::Result;
use crate::persistence::repositories::video_category::VideoCategoryRepoTrait;
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait VideoCategoryFacadeTrait {
    async fn list_categories(&self) -> Result<Vec<VideoCategory>>;
    async fn assign_categories(
        &self,
        video_id: i32,
        category_ids: Vec<i32>,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()>;
    async fn get_selected_categories(&self, video_id: i32) -> Result<Vec<VideoCategorySelected>>;
    async fn add_category(&self, name: String) -> Result<()>;
    async fn delete_category(&self, category_id: i32) -> Result<()>;
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
    async fn list_categories(&self) -> Result<Vec<VideoCategory>> {
        let categories_rows = self.category_repository.fetch_categories().await?;

        let categories = categories_rows.to_mapped_list(VideoCategory::from);

        Ok(categories)
    }

    async fn assign_categories(
        &self,
        video_id: i32,
        category_ids: Vec<i32>,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()> {
        self.category_repository
            .assign_categories(video_id, category_ids, tx)
            .await?;
        Ok(())
    }

    async fn get_selected_categories(&self, video_id: i32) -> Result<Vec<VideoCategorySelected>> {
        let categories = self.category_repository.fetch_categories().await?;
        let assigned_categories = self
            .category_repository
            .get_assigned_categories(video_id)
            .await?;

        let result = categories
            .iter()
            .map(|category| {
                let is_assigned = assigned_categories.contains(category);

                VideoCategorySelected {
                    id: category.id,
                    name: category.name.clone(),
                    selected: is_assigned,
                }
            })
            .collect();

        Ok(result)
    }

    async fn add_category(&self, name: String) -> Result<()> {
        self.category_repository.add_category(name).await?;
        Ok(())
    }

    async fn delete_category(&self, category_id: i32) -> Result<()> {
        self.category_repository
            .delete_category(category_id)
            .await?;
        Ok(())
    }
}
