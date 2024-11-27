#[derive(sqlx::FromRow)]
pub struct Video {
    pub id: u64,
    pub artist_id: u64,
    pub video_visibility: VideoVisibility,
    pub name: String,
    pub file_path: String,
    pub thumbnail_path: String,
    pub description: String,
}

#[allow(clippy::upper_case_acronyms)]
pub enum VideoVisibility {
    ALL,
    REGISTERED,
    PAYING,
}