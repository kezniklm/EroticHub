use crate::persistence::entities::error::MapToDatabaseError;
use crate::persistence::entities::video::{Video, VideoVisibility};
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait VideoRepo {
    #[allow(dead_code)]
    async fn list_videos(&self) -> anyhow::Result<Vec<Video>>;
    async fn save_video(&self, video: Video) -> Result<Video>;
    async fn get_video_by_id(&self, video_id: i32) -> Result<Video>;
}

#[derive(Debug, Clone)]
pub struct PgVideoRepo {
    pg_pool: PgPool,
}

impl PgVideoRepo {
    pub fn new(pg_pool: PgPool) -> Self {
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
            description FROM video ORDER BY id"#
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(result)
    }

    async fn save_video(&self, video: Video) -> Result<Video> {
        let result = sqlx::query_as!(
            Video,
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
        .fetch_one(&self.pg_pool)
        .await?;

        Ok(result)
    }

    async fn get_video_by_id(&self, video_id: i32) -> Result<Video> {
        let result = sqlx::query_as!(
            Video,
            r#"
            SELECT 
            id, 
            artist_id, 
            visibility AS "visibility: VideoVisibility", 
            name, 
            file_path, 
            thumbnail_path, 
            description
            FROM video WHERE id = $1
            "#,
            video_id
        )
        .fetch_one(&self.pg_pool)
        .await
        .db_error("Video doesn't exist")?;

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::tests::setup::AsyncContext;
    use crate::persistence::entities::error::DatabaseError;
    use test_context::test_context;

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn test_save_fetch_video(ctx: &mut AsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());

        let created_video = create_test_video(Some(1), None);

        repo.save_video(created_video.clone())
            .await
            .expect("Failed to save video");
        let video = repo
            .get_video_by_id(created_video.id)
            .await
            .db_error("Failed to fetch video")?;
        assert_eq!(created_video, video);

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn test_save_sequence(ctx: &mut AsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());

        repo.save_video(create_test_video(None, None)).await?;
        repo.save_video(create_test_video(None, None)).await?;

        let video = repo
            .get_video_by_id(2)
            .await
            .db_error("Failed to fetch video")?;
        assert_eq!(
            video.id, 2,
            "Sequence should be used for ID of created video"
        );

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn test_list_videos(ctx: &mut AsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());

        let video1 = repo.save_video(create_test_video(Some(1), None)).await?;
        let video2 = repo.save_video(create_test_video(Some(2), None)).await?;

        let video = repo
            .list_videos()
            .await
            .db_error("Failed to fetch videos")?;
        assert_eq!(video.len(), 2, "Unexpected number of videos");
        assert_eq!(Some(&video1), video.get(0));
        assert_eq!(Some(&video2), video.get(1));

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn video_doesnt_exist(ctx: &mut AsyncContext) {
        let repo = create_repository(ctx.pg_pool.clone());

        let video = repo.get_video_by_id(2).await;
        let expected_err: Result<Video> = Err(DatabaseError::new("Video doesn't exist"));
        assert_eq!(
            video, expected_err,
            "Unexpected error when video doesn't exist"
        );
    }

    fn create_repository(pg_pool: PgPool) -> impl VideoRepo {
        PgVideoRepo { pg_pool }
    }

    fn create_test_video(
        video_id: Option<i32>,
        video_visibility: Option<VideoVisibility>,
    ) -> Video {
        Video {
            id: video_id.unwrap_or(-1),
            artist_id: 1,
            visibility: video_visibility.unwrap_or(VideoVisibility::All),
            name: String::from("John"),
            file_path: String::from("Doe"),
            thumbnail_path: String::from("dummy_path"),
            description: Some(String::from("Description")),
        }
    }

    async fn create_dummy_artist(pg_pool: &PgPool) -> Result<()> {
        sqlx::query!(r#"INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id) VALUES (1, 'John', 'hash', 'email@email.cz', 'path/pic.png', null, null);"#)
            .execute(pg_pool).await?;
        sqlx::query!(
            r#"INSERT INTO artist(id, user_id, description) VALUES (1, 1, 'description')"#
        )
        .execute(pg_pool)
        .await?;
        sqlx::query!(r#"UPDATE artist SET user_id = 1 WHERE user_id = 1"#)
            .execute(pg_pool)
            .await?;

        Ok(())
    }
}
