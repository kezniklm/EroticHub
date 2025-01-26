use crate::business::facades::video::VideoFacadeTrait;
use crate::business::models::error::{AppError, AppErrorKind, MapToAppError};
use crate::business::models::stream::{
    CompoundStreamInfo, LiveStream as LiveStreamDto, LiveStreamStart, StreamStorage,
};
use crate::business::models::video::Video;
use crate::business::Result;
use crate::persistence::entities::error::MapToDatabaseError;
use crate::persistence::entities::stream::{LiveStream, LiveStreamStatus};
use crate::persistence::repositories::stream::StreamRepoTrait;
use crate::streamer;
use crate::streamer::gstreamer_controller::create_streams;
use crate::streamer::types::StreamResolution;
use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;

const NGINX_HLS_URL_KEY: &str = "NGINX_HLS_URL";
const STREAM_PREFIX_KEY: &str = "STREAM_PATH_PREFIX";

lazy_static! {
    static ref STREAM_PREFIX: String =
        dotenvy::var(STREAM_PREFIX_KEY).expect("Stream is wrongly configured");
    static ref PLAYLIST_REGEX: Regex =
        Regex::new(format!(r#"/hls/{}-(\d+)_?\d*.m3u8"#, STREAM_PREFIX.as_str()).as_str()).unwrap();
    static ref TRANSPORT_STREAM_REGEX: Regex =
        Regex::new(format!(r#"/hls/{}-(\d+)_\d*-\d*.ts"#, STREAM_PREFIX.as_str()).as_str())
            .unwrap();
}

#[async_trait]
pub trait StreamFacadeTrait {
    #[allow(dead_code)]
    async fn schedule_stream(&self, video_id: i32, user_id: i32) -> anyhow::Result<()>;
    /// Starts the stream of the given video
    ///
    /// # Returns
    /// `i32` - stream ID of created stream
    async fn start_stream(&self, live_stream: LiveStreamStart, user_id: i32) -> Result<i32>;
    /// Gets live stream by it's ID
    /// TODO: needs validation of user and permissions check!
    ///
    /// # Returns
    /// `LiveStreamDto` - data transfer object of livestream
    async fn get_stream(&self, user_id: i32, stream_id: i32) -> Result<(Video, LiveStreamDto)>;
    async fn stop_stream(&self, user_id: i32, stream_id: i32) -> Result<()>;
    /// Nginx asks for authentication when user tries to access the stream,
    /// Check if user has permissions to view the stream (according to video settings)
    ///
    /// # Params
    /// `stream_url` - e.g. /hls/stream-3.m3u8 (stream-{id}.m3u8)
    async fn authenticate_stream(&self, user_id: i32, stream_url: &str) -> Result<()>;
}

pub struct StreamFacade {
    video_facade: Arc<dyn VideoFacadeTrait + Send + Sync>,
    stream_storage: Arc<StreamStorage>,
    stream_repo: Arc<dyn StreamRepoTrait + Send + Sync>,
}

impl StreamFacade {
    pub fn new(
        video_facade: Arc<dyn VideoFacadeTrait + Send + Sync>,
        stream_storage: Arc<StreamStorage>,
        stream_repo: Arc<dyn StreamRepoTrait + Send + Sync>,
    ) -> Self {
        Self {
            video_facade,
            stream_storage,
            stream_repo,
        }
    }

    fn create_stream(&self, stream_info: Arc<CompoundStreamInfo>) -> Result<String> {
        let stream_url = self.create_stream_url(stream_info.stream_id.clone())?;
        let stream_repo = self.stream_repo.clone();
        let info = stream_info.clone();

        let stream_storage = self.stream_storage.clone();

        thread::spawn::<_, Result<()>>(move || {
            let handles = create_streams(stream_storage, stream_info.clone())
                .app_error("Failed to create streams")?;

            for handle in handles {
                handle.join().app_error("Failed to end the stream")?;
            }

            let runtime = Runtime::new().app_error("Failed to end the stream")?;
            runtime.block_on(Self::mark_stream_as_ended(info, stream_repo))?;
            Ok(())
        });

        Ok(stream_url)
    }

    async fn mark_stream_as_ended(
        stream_info: Arc<CompoundStreamInfo>,
        pg_stream_repo: Arc<dyn StreamRepoTrait + Send + Sync>,
    ) -> Result<()> {
        let stream_id = stream_info
            .stream_id
            .parse()
            .app_error("Stream ID has unexpected format")?;
        pg_stream_repo
            .change_status(stream_id, LiveStreamStatus::Ended)
            .await
            .db_error("Failed to change status of the stream")?;

        Ok(())
    }

    fn create_stream_url(&self, stream_id: String) -> Result<String> {
        let nginx_url =
            dotenvy::var(NGINX_HLS_URL_KEY).app_error("Stream is wrongly configured")?;
        let stream_prefix =
            dotenvy::var(STREAM_PREFIX_KEY).app_error("Stream is wrongly configured")?;
        let url = format!("{nginx_url}{stream_prefix}-{}.m3u8", stream_id);

        Ok(url)
    }
}

#[async_trait]
impl StreamFacadeTrait for StreamFacade {
    async fn schedule_stream(&self, _video_id: i32, _user_id: i32) -> anyhow::Result<()> {
        todo!()
    }

    async fn start_stream(&self, live_stream: LiveStreamStart, user_id: i32) -> Result<i32> {
        let video = self
            .video_facade
            .get_video_entity(live_stream.video_id, Some(user_id))
            .await?;

        let stream_id = self
            .stream_repo
            .add_stream(LiveStream::from(&live_stream))
            .await?;

        let stream_info = CompoundStreamInfo::new(
            stream_id.to_string(),
            video.file_path,
            vec![
                StreamResolution::P360,
                // StreamResolution::P480,
                // StreamResolution::P720,
            ],
        );
        self.create_stream(Arc::new(stream_info))?;
        Ok(stream_id)
    }

    async fn get_stream(&self, user_id: i32, stream_id: i32) -> Result<(Video, LiveStreamDto)> {
        let stream = self
            .stream_repo
            .get_stream(stream_id)
            .await?
            .ok_or(AppError::new(
                "Desired stream doesn't exist",
                AppErrorKind::NotFound,
            ))?;

        let stream_id_str = stream.id.to_string();
        let stream_dto = LiveStreamDto::from_entity(stream, self.create_stream_url(stream_id_str)?);

        let video = self
            .video_facade
            .get_video_model(stream_dto.video_id, Some(user_id))
            .await?;

        Ok((video, stream_dto))
    }

    async fn stop_stream(&self, _user_id: i32, stream_id: i32) -> Result<()> {
        self.stream_storage
            .run_on(&stream_id.to_string(), |stream| {
                let (_info, pipelines) = stream;
                for pipeline in pipelines {
                    streamer::gstreamer_controller::stop_stream(pipeline)
                        .app_error("Failed to stop the stream")?;
                }

                Ok(())
            })?;

        self.stream_repo
            .change_status(stream_id, LiveStreamStatus::Ended)
            .await?;
        Ok(())
    }

    async fn authenticate_stream(&self, _user_id: i32, stream_url: &str) -> Result<()> {
        let err = AppError::new("Failed to parse stream URL", AppErrorKind::AccessDenied);
        let regex;
        if stream_url.ends_with(".m3u8") {
            regex = PLAYLIST_REGEX.deref();
        } else if stream_url.ends_with(".ts") {
            regex = TRANSPORT_STREAM_REGEX.deref();
        } else {
            return Err(err);
        }

        let captures = regex.captures(stream_url).ok_or(err.clone())?;

        let stream_id: i32 = captures
            .get(1)
            .ok_or(err.clone())?
            .as_str()
            .parse()
            .map_err(|_| err)?;

        let _visibility = self.stream_repo.get_visibility(stream_id).await?;
        // TODO: Check permissions!
        Ok(())
    }
}
