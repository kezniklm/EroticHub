use crate::persistence::entities::artist::Artist;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait ArtistRepoTrait: Debug {
    async fn list_artists(&self) -> anyhow::Result<Vec<Artist>>;
}

#[derive(Debug, Clone)]
pub struct ArtistRepository {
    pg_pool: PgPool,
}

impl ArtistRepository {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl ArtistRepoTrait for ArtistRepository {
    async fn list_artists(&self) -> anyhow::Result<Vec<Artist>> {
        let artists = sqlx::query_as!(Artist, "SELECT * FROM artist")
            .fetch_all(&self.pg_pool)
            .await?;

        Ok(artists)
    }
}
