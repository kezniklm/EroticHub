use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserList {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub profile_picture_path: Option<String>,
}
