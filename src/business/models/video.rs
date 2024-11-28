use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum VideoVisibility {
    ALL,
    REGISTERED,
    PAYING,
}

#[derive(MultipartForm)]
pub struct VideoUploadForm {
    #[multipart(limit = "500MB")]
    pub file: TempFile,
}

#[derive(MultipartForm)]
pub struct ThumbnailUploadForm {
    #[multipart(limit = "10MB")]
    pub file: TempFile,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoUploadData {
    pub video_visibility: VideoVisibility,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct TempFileResponse {
    pub temp_file_id: i32,
}
