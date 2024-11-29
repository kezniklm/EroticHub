use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    app: AppConfiguration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfiguration {
    video: VideoConfig,
    thumbnail: Thumbnail,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoConfig {
    pub accepted_mime_type: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thumbnail {
    pub accepted_mime_type: Vec<String>,
}
