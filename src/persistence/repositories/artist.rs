use crate::business::models::artist_detail::ArtistName;
use crate::persistence::entities::artist::Artist;
use crate::persistence::entities::error::MapToDatabaseError;
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Debug;

#[async_trait]
pub trait ArtistRepoTrait: Debug {
    async fn list_artists(&self) -> anyhow::Result<Vec<Artist>>;
    async fn fetch_artists_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<Artist>>;
    async fn fetch_artists_names_by_id(&self, ids: Vec<i32>) -> Result<Vec<ArtistName>>;
    async fn get_artist(
        &self,
        user_id: i32,
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> Result<Artist>;
    async fn make_user_artist(&self, user_id: i32) -> Result<()>;
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

    async fn fetch_artists_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<Artist>> {
        let artists = sqlx::query_as!(
            Artist,
            r#"
            SELECT id, user_id, description
            FROM artist
            WHERE id IN (SELECT unnest($1::integer[]))
            "#,
            &ids
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(artists)
    }

    async fn fetch_artists_names_by_id(&self, ids: Vec<i32>) -> Result<Vec<ArtistName>> {
        let artists = sqlx::query_as!(
            ArtistName,
            r#"
            SELECT artist.id, artist.user_id, user_table.username as name
            FROM artist JOIN user_table ON user_table.id = artist.user_id
            WHERE artist.id IN (SELECT unnest($1::integer[]))
            "#,
            &ids
        )
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

    async fn make_user_artist(&self, user_id: i32) -> Result<()> {
        let mut tx = self.pg_pool.begin().await?;

        let artist_id = sqlx::query!(
            "INSERT INTO artist (user_id) VALUES ($1) RETURNING id",
            user_id
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;

        sqlx::query!(
            "UPDATE user_table SET artist_id = $1 WHERE id = $2",
            artist_id,
            user_id
        )
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
