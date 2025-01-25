use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    pub app: AppConfiguration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfiguration {
    pub video: VideoConfig,
    pub thumbnail: Thumbnail,
    pub stream: Stream,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoConfig {
    pub accepted_mime_type: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thumbnail {
    pub accepted_mime_type: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stream {
    pub resolutions: Vec<String>,
}
