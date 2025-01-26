use crate::persistence::entities::stream::{LiveStream, LiveStreamStatus};
use crate::persistence::entities::video::{Video, VideoVisibility};
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait StreamRepoTrait {
    async fn add_stream(&self, stream: LiveStream) -> Result<i32>;
    async fn change_status(&self, stream_id: i32, status: LiveStreamStatus) -> Result<()>;
    async fn get_stream(&self, stream_id: i32) -> Result<Option<LiveStream>>;
    async fn get_streamed_video(&self, stream_id: i32) -> Result<Video>;
    async fn get_visibility(&self, stream_id: i32) -> Result<VideoVisibility>;
}

pub struct PgStreamRepo {
    pg_pool: PgPool,
}

impl PgStreamRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl StreamRepoTrait for PgStreamRepo {
    async fn add_stream(&self, stream: LiveStream) -> Result<i32> {
        // SQLx doesn't support optional for enum type
        let result = sqlx::query!(
            r#"INSERT INTO live_stream(video_id, start_time, status)
            VALUES ($1, $2, $3) RETURNING live_stream.id"#,
            stream.video_id,
            stream.start_time,
            stream.status as LiveStreamStatus
        )
        .fetch_one(&self.pg_pool)
        .await?;

        Ok(result.id)
    }

    async fn change_status(&self, stream_id: i32, status: LiveStreamStatus) -> Result<()> {
        sqlx::query!(
            "UPDATE live_stream SET status = $1 WHERE id = $2",
            status as LiveStreamStatus,
            stream_id
        )
        .execute(&self.pg_pool)
        .await?;
        Ok(())
    }

    async fn get_stream(&self, stream_id: i32) -> Result<Option<LiveStream>> {
        let stream = sqlx::query_as!(
            LiveStream,
            r#"SELECT id, video_id, start_time, status as "status: LiveStreamStatus"
            FROM live_stream WHERE id = $1"#,
            stream_id
        )
        .fetch_optional(&self.pg_pool)
        .await?;
        Ok(stream)
    }

    async fn get_streamed_video(&self, stream_id: i32) -> Result<Video> {
        let video = sqlx::query_as!(
            Video,
            r#"SELECT 
                video.id,
                artist_id,
                visibility AS "visibility: VideoVisibility",
                name,
                file_path,
                thumbnail_path,
                description 
            FROM video
            JOIN live_stream ON live_stream.video_id = video.id
            WHERE live_stream.id = $1"#,
            stream_id
        )
        .fetch_one(&self.pg_pool)
        .await?;

        Ok(video)
    }

    async fn get_visibility(&self, stream_id: i32) -> Result<VideoVisibility> {
        let record = sqlx::query!(
            r#"
            SELECT visibility AS "visibility: VideoVisibility"
            FROM live_stream JOIN video ON video.id = live_stream.video_id
            WHERE live_stream.id = $1
        "#,
            stream_id
        )
        .fetch_one(&self.pg_pool)
        .await?;

        Ok(record.visibility)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::tests::setup::{AsyncContext, EmptyAsyncContext};
    use crate::persistence::entities::stream::{LiveStream, LiveStreamStatus};
    use crate::persistence::entities::video::{Video, VideoVisibility};
    use crate::persistence::repositories::video::{PgVideoRepo, VideoRepo};
    use chrono::Local;
    use strum::IntoEnumIterator;
    use test_context::test_context;

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn add_get_stream(ctx: &mut AsyncContext) -> Result<()> {
        let video = create_dummy_video(&ctx).await?;
        let stream = create_stream_entity(&video);
        let repo = PgStreamRepo::new(ctx.pg_pool.clone());

        let stream_id = repo.add_stream(stream.clone()).await?;
        let created_stream = repo.get_stream(stream_id).await?;
        assert!(created_stream.is_some(), "Failed to fetch existing stream");
        let created_stream = created_stream.unwrap();
        assert_eq!(created_stream.id, stream_id);
        assert_eq!(created_stream.status, stream.status);
        assert_eq!(
            created_stream.start_time.timestamp_millis(),
            stream.start_time.timestamp_millis()
        );
        assert_eq!(created_stream.video_id, stream.video_id);

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn get_empty(ctx: &mut EmptyAsyncContext) -> Result<()> {
        let repo = PgStreamRepo::new(ctx.pg_pool.clone());

        let created_stream = repo.get_stream(-1).await?;
        assert!(
            created_stream.is_none(),
            "None should be returned for non-existing stream"
        );
        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn change_status(ctx: &mut AsyncContext) -> Result<()> {
        let video = create_dummy_video(&ctx).await?;
        let stream = create_stream_entity(&video);
        let repo = PgStreamRepo::new(ctx.pg_pool.clone());

        let stream_id = repo.add_stream(stream.clone()).await?;

        for status in LiveStreamStatus::iter() {
            repo.change_status(stream_id, status.clone()).await?;
            let stream = repo.get_stream(stream_id).await?.unwrap();
            assert_eq!(
                stream.status, status,
                "Live stream has unexpected status after change"
            );
        }

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn get_streamed_video(ctx: &mut AsyncContext) -> Result<()> {
        let video = create_dummy_video(&ctx).await?;
        let stream = create_stream_entity(&video);
        let repo = PgStreamRepo::new(ctx.pg_pool.clone());

        let stream_id = repo.add_stream(stream.clone()).await?;
        let streamed_video = repo.get_streamed_video(stream_id).await?;

        assert_eq!(
            streamed_video, video,
            "Streamed video doesn't match the created video"
        );

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn get_visibility(ctx: &mut AsyncContext) -> Result<()> {
        let video = create_dummy_video(&ctx).await?;
        let stream = create_stream_entity(&video);
        let repo = PgStreamRepo::new(ctx.pg_pool.clone());

        let stream_id = repo.add_stream(stream.clone()).await?;

        let visibility = repo.get_visibility(stream_id).await?;
        assert_eq!(
            visibility,
            VideoVisibility::All,
            "Returned unexpected video visibility"
        );
        Ok(())
    }

    async fn create_dummy_video(ctx: &AsyncContext) -> Result<Video> {
        let video = Video {
            id: 1,
            artist_id: 1,
            visibility: VideoVisibility::All,
            name: String::from("Test video"),
            file_path: String::from("dummy path"),
            thumbnail_path: String::from("dummy path"),
            description: None,
        };

        let repo = PgVideoRepo::new(ctx.pg_pool.clone());

        let mut tx = ctx.pg_pool.begin().await?;
        let video = repo.save_video(video, &mut tx).await?;
        tx.commit().await?;

        Ok(video)
    }

    fn create_stream_entity(video: &Video) -> LiveStream {
        LiveStream {
            id: -1,
            video_id: video.id,
            start_time: Local::now(),
            status: LiveStreamStatus::Pending,
        }
    }
}
