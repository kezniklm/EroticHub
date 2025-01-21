use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct VideoCategory {
    pub id: i32,
    pub name: String,
}
