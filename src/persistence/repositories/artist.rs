use crate::persistence::entities::artist::Artist;
use crate::persistence::entities::error::MapToDatabaseError;
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Debug;

#[async_trait]
pub trait ArtistRepoTrait: Debug {
    async fn list_artists(&self) -> anyhow::Result<Vec<Artist>>;
    async fn get_artist(
        &self,
        user_id: i32,
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> Result<Artist>;
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
        let artists = sqlx::query_as!(Artist, "SELECT id, user_id, description FROM artist")
            .fetch_all(&self.pg_pool)
            .await?;

        Ok(artists)
    }

    async fn get_artist(
        &self,
        user_id: i32,
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> Result<Artist> {
        let query = sqlx::query_as!(
            Artist,
            "SELECT id, user_id, description FROM artist WHERE user_id = $1",
            user_id
        );

        let artist = match tx {
            None => query.fetch_one(&self.pg_pool).await,
            Some(tx) => query.fetch_one(tx.as_mut()).await,
        }
        .db_error("Failed to fetch the artist")?;

        Ok(artist)
    }
}
