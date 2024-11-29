use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow)]
pub struct LiveStream {
    pub id: i32,
    pub video_id: i32,
    pub start_time: DateTime<Local>,
    pub status: LiveStreamStatus,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "live_stream_status")]
pub enum LiveStreamStatus {
    PENDING,
    RUNNING,
    ENDED,
}