use crate::business::models::error::{AppError, AppErrorKind, MapToAppError};
use crate::business::Result;
use crate::streamer::types::{
    CompoundStreamInfoTrait, PipelinesList, Stream, StreamResolution, StreamStorageTrait, Streams,
};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, MutexGuard};

const RTMP_SERVER_ENV: &str = "RTMP_SERVER";
const STREAM_PATH_PREFIX_KEY: &str = "STREAM_PATH_PREFIX";

#[derive(Clone)]
pub struct StreamStorage {
    streams: Streams,
}

impl Default for StreamStorage {
    fn default() -> Self {
        StreamStorage {
            streams: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl StreamStorage {
    fn get_index(streams: &MutexGuard<Vec<Stream>>, stream_id: &str) -> Option<usize> {
        let position = streams
            .iter()
            .position(|(stream, _pipeline)| stream.get_stream_id() == stream_id);

        position
    }

    pub fn run_on<F: FnOnce(&Stream) -> anyhow::Result<()>>(
        &self,
        stream_id: &str,
        fnc: F,
    ) -> Result<()> {
        let streams = self.streams.lock().unwrap();
        let position = Self::get_index(&streams, stream_id);

        if let Some(position) = position {
            if let Some(stream) = streams.get(position) {
                return fnc(stream).app_error("Failed to run an action on stream");
            }
        }
        Err(AppError::new("Stream not found", AppErrorKind::NotFound))
    }
}

impl StreamStorageTrait for StreamStorage {
    fn push(&self, stream: Arc<dyn CompoundStreamInfoTrait>, pipeline: PipelinesList) {
        let mut streams = self.streams.lock().unwrap();
        streams.push((stream, pipeline));
    }

    fn remove(&self, stream_id: &str) {
        let mut streams = self.streams.lock().unwrap();
        let position = Self::get_index(&streams, stream_id);
        match position {
            None => (),
            Some(position) => {
                streams.remove(position);
            }
        }
    }

    fn size(&self) -> usize {
        let streams = self.streams.lock().unwrap();
        streams.len()
    }
}

#[derive(Clone)]
pub struct CompoundStreamInfo {
    pub stream_id: String,
    pub video_path: String,
    pub streams: Vec<StreamResolution>,
}

impl CompoundStreamInfo {
    pub fn new(
        stream_id: String,
        video_path: String,
        streams: Vec<StreamResolution>,
    ) -> CompoundStreamInfo {
        CompoundStreamInfo {
            stream_id,
            video_path,
            streams,
        }
    }
}

impl CompoundStreamInfoTrait for CompoundStreamInfo {
    fn compose_rtmp_url(&self, resolution: StreamResolution) -> String {
        let rtmp_server_path =
            dotenvy::var(RTMP_SERVER_ENV).expect("RTMP server path is not defined");
        let stream_path_prefix =
            dotenvy::var(STREAM_PATH_PREFIX_KEY).expect("Stream path prefix is not defined");
        format!(
            "{}/{}-{}_{}",
            rtmp_server_path,
            stream_path_prefix,
            self.stream_id,
            resolution.as_str()
        )
    }

    fn get_stream_id(&self) -> String {
        self.stream_id.clone()
    }

    fn get_video_path(&self) -> String {
        self.video_path.clone()
    }

    fn get_resolutions(&self) -> &Vec<StreamResolution> {
        &self.streams
    }
}

#[derive(Serialize, Deserialize)]
pub struct LiveStreamSchedule {
    pub video_id: i32,
    pub start_time: DateTime<Local>,
}

#[derive(Serialize, Deserialize)]
pub struct LiveStreamStart {
    pub video_id: i32,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum LiveStreamStatus {
    Pending,
    Running,
    Ended,
}

#[derive(Serialize, Deserialize)]
pub struct LiveStream {
    pub id: i32,
    pub video_id: i32,
    pub start_time: DateTime<Local>,
    pub status: LiveStreamStatus,
    pub stream_url: String,
}
