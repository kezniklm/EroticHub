use crate::configuration::models::Configuration;
use askama_actix::Template;
use std::sync::Arc;

#[derive(Template)]
#[template(path = "video/upload/video_upload.html")]
pub struct VideoUploadTemplate<V: Template, T: Template> {
    pub video_input: V,
    pub thumbnail_input: T,
}

#[derive(Template)]
#[template(path = "video/upload/inputs/upload_video.html")]
pub struct VideoUploadInputTemplate {
    pub accepted_mimetype: String,
}

impl VideoUploadInputTemplate {
    pub fn new(config: Arc<Configuration>) -> Self {
        let accepted_mimetype = config.app.video.accepted_mime_type.join(",");
        Self { accepted_mimetype }
    }
}

#[derive(Template)]
#[template(path = "video/upload/inputs/upload_thumbnail.html")]
pub struct ThumbnailUploadInputTemplate {
    pub accepted_mimetype: String,
}

impl ThumbnailUploadInputTemplate {
    pub fn new(config: Arc<Configuration>) -> Self {
        let accepted_mimetype = config.app.thumbnail.accepted_mime_type.join(",");
        Self { accepted_mimetype }
    }
}

#[derive(Template)]
#[template(path = "video/upload/inputs/preview_video.html")]
pub struct VideoPreviewTemplate {
    pub temp_file_id: Option<i32>,
    pub file_path: String,
}

#[derive(Template)]
#[template(path = "video/upload/inputs/preview_thumbnail.html")]
pub struct ThumbnailPreviewTemplate {
    pub temp_file_id: Option<i32>,
    pub file_path: String,
}
