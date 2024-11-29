use crate::business::facades::video::VideoFacadeTrait;
use crate::business::models::stream::{CompoundStreamInfo, LiveStreamStart};
use crate::persistence::entities::stream::{LiveStream, LiveStreamStatus};
use crate::persistence::repositories::stream::{StreamRepoTrait};
use crate::streamer::gstreamer_controller::create_streams;
use crate::streamer::types::{StreamResolution, StreamStorageTrait};
use async_trait::async_trait;
use std::sync::Arc;
use log::{error, info};

const NGINX_HLS_URL_KEY: &str = "NGINX_HLS_URL";
const STREAM_PREFIX_KEY: &str = "STREAM_PATH_PREFIX";

#[async_trait]
pub trait StreamFacadeTrait {
    async fn schedule_stream(&self, video_id: i32, user_id: i32) -> anyhow::Result<()>;
    async fn start_stream(&self, live_stream: LiveStreamStart, user_id: i32) -> anyhow::Result<String>;
}

pub struct StreamFacade {
    video_facade: Arc<dyn VideoFacadeTrait + Send + Sync>,
    stream_storage: Arc<dyn StreamStorageTrait + Send + Sync>,
    stream_repo: Arc<dyn StreamRepoTrait + Send + Sync>,
}

impl StreamFacade {
    pub fn new(
        video_facade: Arc<dyn VideoFacadeTrait + Send + Sync>,
        stream_storage: Arc<dyn StreamStorageTrait + Send + Sync>,
        stream_repo: Arc<dyn StreamRepoTrait + Send + Sync>, ) -> Self {
        Self {
            video_facade,
            stream_storage,
            stream_repo,
        }
    }

    fn create_stream(&self, stream_info: CompoundStreamInfo) -> anyhow::Result<String> {
        let stream_url = self.create_stream_url(stream_info.stream_id.clone())?;
        let handles = create_streams(self.stream_storage.clone(), Arc::new(stream_info.clone()))?;
        let stream_repo = self.stream_repo.clone();
        actix_rt::spawn(async move {
            for handle in handles {
                match handle.join() {
                    Ok(_) => (),
                    Err(_) => {
                        error!("Error occurred during the stream!");
                    }
                }
            }
            match Self::mark_stream_as_ended(stream_info.clone(), stream_repo).await {
                Ok(_) => (),
                Err(_) => {
                    error!("Failed to mark the stream as ended!");
                }
            }
        });
        Ok(stream_url)
    }

    async fn mark_stream_as_ended(stream_info: CompoundStreamInfo, pg_stream_repo: Arc<dyn StreamRepoTrait + Send + Sync>) -> anyhow::Result<()> {
        pg_stream_repo.change_status(stream_info.stream_id.parse()?, LiveStreamStatus::ENDED).await?;

        Ok(())
    }

    fn create_stream_url(&self, stream_id: String) -> anyhow::Result<String> {
        let nginx_url = dotenvy::var(NGINX_HLS_URL_KEY)?;
        let stream_prefix = dotenvy::var(STREAM_PREFIX_KEY)?;
        let url = format!("{nginx_url}{stream_prefix}-{}.m3u8", stream_id);

        Ok(url)
    }
}

#[async_trait]
impl StreamFacadeTrait for StreamFacade {
    async fn schedule_stream(&self, video_id: i32, user_id: i32) -> anyhow::Result<()> {
        todo!()
    }

    async fn start_stream(&self, live_stream: LiveStreamStart, user_id: i32) -> anyhow::Result<String> {
        let video = self.video_facade.get_video_entity(live_stream.video_id, user_id).await?;

        let stream_id = self.stream_repo.add_stream(LiveStream::from(&live_stream)).await?;

        let stream_info = CompoundStreamInfo::new(
            stream_id.to_string(),
            video.file_path,
            vec![
                StreamResolution::P360,
                StreamResolution::P480,
                StreamResolution::P720,
            ],
        );
        let stream_url = self.create_stream(stream_info)?;
        Ok(stream_url)
    }
}



