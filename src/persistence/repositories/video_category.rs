use crate::persistence::entities::video_category::VideoCategory;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait VideoCategoryRepoTrait: Debug {
    async fn fetch_categories(&self) -> anyhow::Result<Vec<VideoCategory>>;
}

#[derive(Debug, Clone)]
pub struct VideoCategoryRepository {
    pg_pool: PgPool,
}

impl VideoCategoryRepository {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl VideoCategoryRepoTrait for VideoCategoryRepository {
    async fn fetch_categories(&self) -> anyhow::Result<Vec<VideoCategory>> {
        let categories = sqlx::query_as!(VideoCategory, "SELECT id, name FROM video_category")
            .fetch_all(&self.pg_pool)
            .await?;

        Ok(categories)
    }
}
