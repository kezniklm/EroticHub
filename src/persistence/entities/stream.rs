use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow)]
pub struct LiveStream {
    pub id: i32,
    pub video_id: i32,
    pub start_time: DateTime<Local>,
    pub status: LiveStreamStatus,
}

/// This struct should be used only for insert operations!
/// 
/// Fields `start_time` and `status` have defined defaults in database, so they don't need to
/// be defined!
#[derive(sqlx::FromRow)]
pub struct LiveStreamInsert {
    pub video_id: i32,
    pub start_time: Option<DateTime<Local>>,
    pub status: Option<LiveStreamStatus>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "live_stream_status")]
pub enum LiveStreamStatus {
    PENDING,
    RUNNING,
    ENDED,
}