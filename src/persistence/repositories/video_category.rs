use crate::persistence::entities::video_category::VideoCategory;
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Debug;

#[async_trait]
pub trait VideoCategoryRepoTrait: Debug {
    async fn fetch_categories(&self) -> Result<Vec<VideoCategory>>;
    async fn assign_categories(
        &self,
        video_id: i32,
        category_ids: Vec<i32>,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()>;
    async fn delete_assigned_categories(
        &self,
        video_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()>;

    /// Returns assigned categories to specified video
    async fn get_assigned_categories(&self, video_id: i32) -> Result<Vec<VideoCategory>>;
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
    async fn fetch_categories(&self) -> Result<Vec<VideoCategory>> {
        let categories = sqlx::query_as!(VideoCategory, "SELECT id, name FROM video_category")
            .fetch_all(&self.pg_pool)
            .await?;

        Ok(categories)
    }

    async fn assign_categories(
        &self,
        video_id: i32,
        category_ids: Vec<i32>,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()> {
        let query = sqlx::query!(
            r#"INSERT INTO video_category_video(video_id, category_id) 
               SELECT $1, UNNEST($2::integer[])"#,
            video_id,
            &category_ids
        );

        match tx {
            None => {
                let mut tx = self.pg_pool.begin().await?;
                self.delete_assigned_categories(video_id, Some(&mut tx))
                    .await?;
                query.execute(tx.as_mut()).await?;

                tx.commit().await?;
            }
            Some(tx) => {
                self.delete_assigned_categories(video_id, Some(tx)).await?;
                query.execute(tx.as_mut()).await?;
            }
        }

        Ok(())
    }

    async fn delete_assigned_categories(
        &self,
        video_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()> {
        let query = sqlx::query!(
            "DELETE FROM video_category_video WHERE video_id = $1",
            video_id
        );

        match tx {
            None => query.execute(&self.pg_pool).await?,
            Some(tx) => query.execute(tx.as_mut()).await?,
        };
        Ok(())
    }

    async fn get_assigned_categories(&self, video_id: i32) -> Result<Vec<VideoCategory>> {
        let result = sqlx::query_as!(
            VideoCategory,
            r#"
            SELECT id, name FROM video_category
            JOIN video_category_video ON video_category.id = category_id
            WHERE video_id = $1 
        "#,
            video_id
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(result)
    }
}
