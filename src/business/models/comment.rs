use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommentModel {
    pub id: i32,
    pub user_id: i32,
    pub video_id: i32,
    pub created_at: NaiveDateTime,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommentCreateModel {
    pub user_id: i32,
    pub video_id: i32,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommentUserModel {
    pub id: i32,
    pub user_id: i32,
    pub comment_content: String,
    pub created_at: String,
    pub profile_picture_path: Option<String>,
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FetchCommentsOffset {
    pub offset: Option<i64>,
}
