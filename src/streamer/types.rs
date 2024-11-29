use gstreamer::Pipeline;
use std::sync::Arc;

pub type PipelinesList = Vec<Arc<Pipeline>>;

pub trait StreamStorageTrait: Send + Sync {
    fn push(&self, stream: Arc<dyn CompoundStreamInfoTrait>, pipeline: PipelinesList);
    fn remove(&self, stream_id: &str);
    fn size(&self) -> usize;
}

pub trait CompoundStreamInfoTrait: Send + Sync {
    fn compose_rtmp_url(&self, resolution: StreamResolution) -> String;
    fn get_stream_id(&self) -> String;
    fn get_video_path(&self) -> String;
    fn get_resolutions(&self) -> &Vec<StreamResolution>;
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
