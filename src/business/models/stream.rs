use crate::streamer::types::{
    CompoundStreamInfoTrait, PipelinesList, StreamResolution, StreamStorageTrait,
};
use std::sync::{Arc, Mutex};

const RTMP_SERVER_ENV: &str = "RTMP_SERVER";
const STREAM_PATH_PREFIX_KEY: &str = "STREAM_PATH_PREFIX";

type Streams = Arc<Mutex<Vec<(Arc<dyn CompoundStreamInfoTrait>, PipelinesList)>>>;
#[derive(Clone)]
pub struct StreamStorage {
    streams: Streams,
}

impl StreamStorage {
    pub fn new() -> StreamStorage {
        StreamStorage {
            streams: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl StreamStorageTrait for StreamStorage {
    fn push(&self, stream: Arc<dyn CompoundStreamInfoTrait>, pipeline: PipelinesList) {
        let mut streams = self.streams.lock().unwrap();
        streams.push((stream, pipeline));
    }

    fn remove(&self, stream_id: &str) {
        let mut streams = self.streams.lock().unwrap();
        let position = streams
            .iter()
            .position(|(stream, _pipeline)| stream.get_stream_id() == stream_id);
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
    fn compose_stream_url(&self, resolution: StreamResolution) -> String {
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
