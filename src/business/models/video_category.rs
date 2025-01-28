use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct VideoCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct VideoCategorySelected {
    pub id: i32,
    pub name: String,
    pub selected: bool,
}
