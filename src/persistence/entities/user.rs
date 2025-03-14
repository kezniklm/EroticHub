use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: Option<String>,
    pub email: String,
    pub profile_picture_path: Option<String>,
    pub artist_id: Option<i32>,
    pub paying_member_id: Option<i32>,
    pub is_admin: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserName {
    pub id: i32,
    pub username: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LikedVideo {
    pub user_id: i32,
    pub video_id: i32,
}
