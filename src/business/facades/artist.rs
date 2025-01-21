use crate::business::mappers::generic::ToMappedList;
use crate::business::models::artist_detail::{ArtistDetail, ArtistName};
use crate::persistence::repositories::artist::ArtistRepoTrait;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait ArtistFacadeTrait {
    async fn list_artists(&self) -> anyhow::Result<Vec<ArtistDetail>>;
    async fn get_artists_names_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<ArtistName>>;
}

#[derive(Debug, Clone)]
pub struct ArtistFacade {
    artist_repository: Arc<dyn ArtistRepoTrait + Send + Sync>,
}

impl ArtistFacade {
    pub fn new(artist_repository: Arc<dyn ArtistRepoTrait + Send + Sync>) -> Self {
        Self { artist_repository }
    }
}

#[async_trait]
impl ArtistFacadeTrait for ArtistFacade {
    async fn list_artists(&self) -> anyhow::Result<Vec<ArtistDetail>> {
        let artists = self.artist_repository.list_artists().await?;

        let artists_details = artists.to_mapped_list(ArtistDetail::from);

        Ok(artists_details)
    }

    async fn get_artists_names_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<ArtistName>> {
        let artist_names = self
            .artist_repository
            .fetch_artists_names_by_id(ids)
            .await?;

        Ok(artist_names)
    }
}
