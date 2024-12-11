use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommentModel {
    pub id: i32,
    pub user_id: i32,
    pub video_id: i32,
    pub content: String,
}
