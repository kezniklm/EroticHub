use crate::business::facades::artist::ArtistFacadeTrait;
use crate::persistence::repositories::video::VideoRepo;
use std::sync::Arc;

pub struct PatchVideoValidationContext {
    pub pg_video_repo: Arc<dyn VideoRepo + Send + Sync>,
    pub artist_facade: Arc<dyn ArtistFacadeTrait + Send + Sync>,
    pub user_id: i32,
    pub video_id: i32,
}
