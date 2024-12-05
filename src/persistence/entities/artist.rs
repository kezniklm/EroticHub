use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Artist {
    pub id: i32,
    pub user_id: i32,
    pub description: String,
}
