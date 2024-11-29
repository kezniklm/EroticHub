use crate::business::models::stream::LiveStreamStart;
use crate::persistence::entities::stream::{LiveStreamInsert, LiveStreamStatus};

impl From<&LiveStreamStart> for LiveStreamInsert {
    fn from(value: &LiveStreamStart) -> Self {
        Self {
            video_id: value.video_id,
            start_time: None,
            status: Some(LiveStreamStatus::RUNNING),
        }
    }
}