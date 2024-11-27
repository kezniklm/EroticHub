use crate::persistence::entities::video::{Video, VideoVisibility};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

#[async_trait]
pub trait VideoRepo {
    async fn list_videos(&self) -> anyhow::Result<Vec<Video>>;
    async fn save_video(&self, video: Video) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct PgVideoRepo {
    pg_pool: Arc<PgPool>,
}

impl PgVideoRepo {
    pub fn new(pg_pool: Arc<PgPool>) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl VideoRepo for PgVideoRepo {
    async fn list_videos(&self) -> anyhow::Result<Vec<Video>> {
        let result = sqlx::query_as!(
            Video,
            r#"SELECT
            id,
            artist_id,
            visibility AS "visibility: VideoVisibility",
            name,
            file_path,
            thumbnail_path,
            description FROM video"#
        )
        .fetch_all(self.pg_pool.as_ref())
        .await?;

        Ok(result)
    }

    async fn save_video(&self, video: Video) -> anyhow::Result<()> {
        let result = sqlx::query!(
            r#"
            INSERT INTO video(
                artist_id,
                name,
                file_path,
                thumbnail_path,
                description,
                visibility
            ) VALUES ($1, $2, $3, $4, $5, $6) 
            RETURNING id, artist_id, visibility AS "visibility: VideoVisibility",
            name, file_path, thumbnail_path, description 
        "#,
            video.artist_id,
            video.name,
            video.file_path,
            video.thumbnail_path,
            video.description,
            video.visibility as VideoVisibility
        )
        .fetch_one(self.pg_pool.as_ref());

        Ok(())
    }
}
