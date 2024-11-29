use crate::persistence::entities::stream::{LiveStream, LiveStreamStatus};
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait StreamRepoTrait {
    async fn add_stream(&self, stream: LiveStream) -> anyhow::Result<i32>;
    async fn change_status(&self, stream_id: i32, status: LiveStreamStatus) -> anyhow::Result<()>;
}

pub struct PgStreamRepo {
    pg_pool: PgPool,
}

impl PgStreamRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool,
        }
    }
}

#[async_trait]
impl StreamRepoTrait for PgStreamRepo {
    async fn add_stream(&self, stream: LiveStream) -> anyhow::Result<i32> {
        // SQLx doesn't support optional for enum type
        let result = sqlx::query!(r#"INSERT INTO live_stream(video_id, start_time, status)
            VALUES ($1, $2, $3) RETURNING live_stream.id"#,
                stream.video_id, stream.start_time, stream.status as LiveStreamStatus)
            .fetch_one(&self.pg_pool).await?;

        Ok(result.id)
    }

    async fn change_status(&self, stream_id: i32, status: LiveStreamStatus) -> anyhow::Result<()> {
        sqlx::query!("UPDATE live_stream SET status = $1 WHERE id = $2", status as LiveStreamStatus, stream_id)
            .execute(&self.pg_pool).await?;
        Ok(())
    }
}