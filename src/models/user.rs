use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub profile_picture_path: Option<String>,
    pub artist_id: Option<i32>,
    pub paying_member_id: Option<i32>,
}
