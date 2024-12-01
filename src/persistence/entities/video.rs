use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow)]
pub struct Video {
    pub id: i32,
    pub artist_id: i32,
    pub visibility: VideoVisibility,
    pub name: String,
    pub file_path: String,
    pub thumbnail_path: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "visibility_type", rename_all = "UPPERCASE")]
pub enum VideoVisibility {
    All,
    Registered,
    Paying,
}
