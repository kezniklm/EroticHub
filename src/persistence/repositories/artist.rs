use crate::business::models::artist_detail::ArtistName;
use crate::persistence::entities::artist::Artist;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait ArtistRepoTrait: Debug {
    async fn list_artists(&self) -> anyhow::Result<Vec<Artist>>;
    async fn fetch_artists_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<Artist>>;
    async fn fetch_artists_names_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<ArtistName>>;
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

    async fn fetch_artists_names_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<ArtistName>> {
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
}
