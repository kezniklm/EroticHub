use crate::persistence::entities::error::MapToDatabaseError;
use crate::persistence::entities::video::{PatchVideo, Video, VideoVisibility};
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};

#[async_trait]
pub trait VideoRepo {
    #[allow(dead_code)]
    async fn list_videos(&self) -> anyhow::Result<Vec<Video>>;
    async fn save_video(&self, video: Video) -> Result<Video>;
    async fn patch_video(&self, video: PatchVideo) -> Result<Video>;
    async fn delete_video(&self, video_id: i32, user_id: i32) -> Result<bool>;
    async fn get_video_by_id(&self, video_id: i32) -> Result<Option<Video>>;
}

#[derive(Debug, Clone)]
pub struct PgVideoRepo {
    pg_pool: PgPool,
}

impl PgVideoRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
    async fn remove_old_video<'a>(
        video_id: i32,
        transaction: &mut Transaction<'a, Postgres>,
    ) -> Result<()> {
        let old_path = sqlx::query!("SELECT file_path FROM video WHERE id = $1", video_id)
            .fetch_one(transaction.as_mut())
            .await?;
        tokio::fs::remove_file(old_path.file_path)
            .await
            .db_error("Failed to remove old file")?;
        Ok(())
    }

    async fn remove_old_thumbnail<'a>(
        video_id: i32,
        transaction: &mut Transaction<'a, Postgres>,
    ) -> Result<()> {
        let old_path = sqlx::query!("SELECT thumbnail_path FROM video WHERE id = $1", video_id)
            .fetch_one(transaction.as_mut())
            .await?;
        tokio::fs::remove_file(old_path.thumbnail_path)
            .await
            .db_error("Failed to remove old file")?;
        Ok(())
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

    async fn patch_video(&self, video: PatchVideo) -> Result<Video> {
        let mut transaction = self.pg_pool.begin().await?;
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"UPDATE video SET "#);

        let mut first = true;

        if let Some(artist_id) = video.artist_id {
            if !first {
                query.push(",");
            };
            query.push(" artist_id = ");
            query.push_bind(artist_id);

            first = false;
        }

        if let Some(name) = video.name {
            if !first {
                query.push(",");
            };

            query.push(" name = ");
            query.push_bind(name);

            first = false;
        }

        if let Some(ref file_path) = video.file_path {
            if !first {
                query.push(",");
            };

            query.push(" file_path = ");
            query.push_bind(file_path);

            first = false;
        }

        if let Some(ref thumbnail_path) = video.thumbnail_path {
            if !first {
                query.push(",");
            };

            query.push(" thumbnail_path = ");
            query.push_bind(thumbnail_path);

            first = false;
        }

        if let Some(description) = video.description {
            if !first {
                query.push(",");
            };

            query.push(" description = ");
            query.push_bind(description);
        }

        query.push(", visibility = ");
        query.push_bind(video.visibility);

        query.push(" WHERE id = ");
        query.push_bind(video.id);
        query.push(" RETURNING *");

        if video.file_path.is_some() {
            Self::remove_old_video(video.id, &mut transaction).await?;
        }

        if video.thumbnail_path.is_some() {
            Self::remove_old_thumbnail(video.id, &mut transaction).await?;
        }

        let result: Video = query
            .build_query_as()
            .fetch_one(transaction.as_mut())
            .await?;
        transaction.commit().await?;

        Ok(result)
    }

    async fn delete_video(&self, video_id: i32, user_id: i32) -> Result<bool> {
        let mut transaction = self.pg_pool.begin().await?;

        Self::remove_old_video(video_id, &mut transaction).await?;
        Self::remove_old_thumbnail(video_id, &mut transaction).await?;

        let result = sqlx::query!(
            "DELETE FROM video WHERE id = $1 AND artist_id = $2",
            video_id,
            user_id
        )
        .execute(transaction.as_mut())
        .await?;

        if result.rows_affected() == 0 {
            return Ok(false);
        }

        transaction.commit().await?;
        Ok(true)
    }

    async fn get_video_by_id(&self, video_id: i32) -> Result<Option<Video>> {
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
        .fetch_optional(&self.pg_pool)
        .await
        .db_error("Video doesn't exist")?;

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::tests::setup::EmptyAsyncContext;
    use test_context::test_context;

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_save_fetch_video(ctx: &mut EmptyAsyncContext) -> Result<()> {
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
        assert_eq!(Some(created_video), video);

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_save_sequence(ctx: &mut EmptyAsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());

        repo.save_video(create_test_video(None, None)).await?;
        repo.save_video(create_test_video(None, None)).await?;

        let video = repo.get_video_by_id(2).await?.unwrap();
        assert_eq!(
            video.id, 2,
            "Sequence should be used for ID of created video"
        );

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_list_videos(ctx: &mut EmptyAsyncContext) -> Result<()> {
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
        assert_eq!(Some(&video1), video.first());
        assert_eq!(Some(&video2), video.get(1));

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn video_doesnt_exist(ctx: &mut EmptyAsyncContext) -> Result<()> {
        let repo = create_repository(ctx.pg_pool.clone());

        let video = repo.get_video_by_id(2).await?;
        assert_eq!(video, None, "Repository returned unexpected result");

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn patch_video(ctx: &mut EmptyAsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());
        let video1 = repo.save_video(create_test_video(Some(1), None)).await?;

        let edited_video = PatchVideo {
            id: video1.id,
            artist_id: None,
            visibility: VideoVisibility::Paying,
            name: Some(String::from("John2")),
            file_path: None,
            thumbnail_path: None,
            description: Some(String::from("Description2")),
        };
        let updated_video = repo.patch_video(edited_video).await?;
        assert_eq!(updated_video.visibility, VideoVisibility::Paying);
        assert_eq!(updated_video.name, String::from("John2"));
        assert_eq!(
            updated_video.description,
            Some(String::from("Description2"))
        );

        Ok(())
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
