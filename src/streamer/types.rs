pub struct StreamConfig {
    pub stream_id: String,
    pub video_path: String,
    pub quality: StreamQuality,
}

impl StreamConfig {
    // TODO: Load from some configuration file
    pub fn compose_stream_url(&self) -> String {
        format!("rtmp://localhost/hls/stream-{}_{}", self.stream_id, self.quality.as_str())
    }
}

pub enum StreamQuality {
    Q360,
    Q720,
    Q1080,
}

impl StreamQuality {
    fn as_str(&self) -> &'static str {
        match &self {
            StreamQuality::Q360 => { "360" }
            StreamQuality::Q720 => { "720" }
            StreamQuality::Q1080 => { "1080" }
        }
    }
}
