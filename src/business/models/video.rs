use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum VideoVisibility {
    All,
    Registered,
    Paying,
}

impl Display for VideoVisibility {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoVisibility::All => write!(f, "ALL"),
            VideoVisibility::Registered => write!(f, "REGISTERED"),
            VideoVisibility::Paying => write!(f, "PAYING"),
        }
    }
}

#[derive(MultipartForm)]
pub struct VideoUploadForm {
    #[multipart(limit = "1GB")]
    pub file: TempFile,
}

#[derive(MultipartForm)]
pub struct ThumbnailUploadForm {
    #[multipart(limit = "10MB")]
    pub file: TempFile,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct VideoUploadReq {
    #[validate(length(min = 3, max = 128))]
    pub name: String,
    pub video_visibility: VideoVisibility,
    pub category_ids: Vec<i32>,
    #[validate(range(min = 1))]
    pub temp_thumbnail_id: i32,
    #[validate(range(min = 1))]
    pub temp_video_id: i32,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct VideoEditReq {
    #[validate(length(min = 3, max = 128))]
    pub name: Option<String>,
    pub video_visibility: VideoVisibility,
    pub category_ids: Vec<i32>,
    #[validate(range(min = 1))]
    pub temp_thumbnail_id: Option<i32>,
    #[validate(range(min = 1))]
    pub temp_video_id: Option<i32>,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TempFileResponse {
    pub temp_file_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Video {
    pub id: i32,
    pub artist_id: i32,
    pub video_visibility: VideoVisibility,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoList {
    pub id: i32,
    pub artist_id: i32,
    pub artist_name: String,
    pub thumbnail_path: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditVideoTemplateModel {
    pub id: i32,
    pub video_visibility: VideoVisibility,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetVideoByIdReq {
    pub id: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FetchVideoByFilters {
    pub offset: Option<i32>,
    pub ord: Option<String>,
    pub filter: Option<String>,
}
