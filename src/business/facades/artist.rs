use crate::business::mappers::generic::ToMappedList;
use crate::business::models::artist_detail::ArtistDetail;
use crate::persistence::repositories::artist::{ArtistRepoTrait, ArtistRepository};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait ArtistFacadeTrait {
    async fn list_artists(&self) -> anyhow::Result<Vec<ArtistDetail>>;
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
}
