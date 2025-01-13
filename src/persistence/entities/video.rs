use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct Video {
    pub id: i32,
    pub artist_id: i32,
    pub visibility: VideoVisibility,
    pub name: String,
    pub file_path: String,
    pub thumbnail_path: String,
    pub description: Option<String>,
}

pub struct PatchVideo {
    pub id: i32,
    pub artist_id: Option<i32>,
    pub visibility: VideoVisibility,
    pub name: Option<String>,
    pub file_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "visibility_type", rename_all = "UPPERCASE")]
pub enum VideoVisibility {
    All,
    Registered,
    Paying,
}
