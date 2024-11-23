use gstreamer::Pipeline;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct CompoundStreamInfo {
    pub stream_id: String,
    pub video_path: String,
    pub streams: Vec<StreamInfo>,
}

impl CompoundStreamInfo {
    pub fn new(stream_id: String, video_path: String, streams: Vec<StreamInfo>) -> CompoundStreamInfo {
        CompoundStreamInfo {
            stream_id,
            video_path,
            streams,
        }
    }

    // TODO: Load from some configuration file
    pub fn compose_stream_url(&self, quality: StreamResolution) -> String {
        format!(
            "rtmp://localhost/hls/stream-{}_{}",
            self.stream_id,
            quality.as_str()
        )
    }
}

#[derive(Clone)]
pub struct StreamInfo {
    pub resolution: StreamResolution,
}

impl StreamInfo {
    pub fn new(resolution: StreamResolution) -> StreamInfo {
        StreamInfo {
            resolution
        }
    }
}

#[derive(Clone)]
pub enum StreamResolution {
    P360,
    P480,
    P720,
}

impl StreamResolution {
    fn as_str(&self) -> &'static str {
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

#[derive(Clone)]
pub struct StreamStorage {
    streams: Arc<Mutex<Vec<(CompoundStreamInfo, Vec<Arc<Pipeline>>)>>>,
}

impl StreamStorage {
    pub fn new() -> StreamStorage {
        StreamStorage {
            streams: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn push(&self, stream: CompoundStreamInfo, pipeline: Vec<Arc<Pipeline>>) {
        let mut streams = self.streams.lock();
        streams.await.push((stream, pipeline));
    }

    pub async fn remove(&self, stream_id: String) {
        let mut streams = self.streams.lock();
        let mut mutex = streams.await;
        let position = mutex
            .iter()
            .position(|(stream, pipeline)| stream.stream_id == stream_id);
        match position {
            None => (),
            Some(position) => {
                mutex.remove(position);
            }
        }
    }

    pub async fn size(&self) -> usize {
        let streams = self.streams.lock();
        streams.await.len()
    }
}
