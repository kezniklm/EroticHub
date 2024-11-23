use gstreamer::Pipeline;
use std::sync::{Arc, Mutex};

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

    // TODO: Load from some configuration file
    pub fn compose_stream_url(&self, resolution: StreamResolution) -> String {
        format!(
            "rtmp://localhost/hls/stream-{}_{}",
            self.stream_id,
            resolution.as_str()
        )
    }
}

#[derive(Clone)]
pub enum StreamResolution {
    P360,
    P480,
    P720,
}

impl StreamResolution {
    pub fn as_str(&self) -> &'static str {
        match &self {
            StreamResolution::P360 => "360",
            StreamResolution::P480 => "480",
            StreamResolution::P720 => "720",
        }
    }

    pub fn get_resolution(&self) -> (u32, u32) {
        match &self {
            StreamResolution::P360 => (640, 360),
            StreamResolution::P480 => (854, 480),
            StreamResolution::P720 => (1280, 720),
        }
    }
}

type PipelinesList = Vec<Arc<Pipeline>>;
#[derive(Clone)]
pub struct StreamStorage {
    streams: Arc<Mutex<Vec<(CompoundStreamInfo, PipelinesList)>>>,
}

impl StreamStorage {
    pub fn new() -> StreamStorage {
        StreamStorage {
            streams: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push(&self, stream: CompoundStreamInfo, pipeline: PipelinesList) {
        let mut streams = self.streams.lock().unwrap();
        streams.push((stream, pipeline));
    }

    pub fn remove(&self, stream_id: String) {
        let mut streams = self.streams.lock().unwrap();
        let position = streams
            .iter()
            .position(|(stream, _pipeline)| stream.stream_id == stream_id);
        match position {
            None => (),
            Some(position) => {
                streams.remove(position);
            }
        }
    }

    pub fn size(&self) -> usize {
        let streams = self.streams.lock().unwrap();
        streams.len()
    }
}
