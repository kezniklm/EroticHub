use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ArtistDetail {
    pub id: i32,
    pub user_id: i32,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ArtistName {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
}
