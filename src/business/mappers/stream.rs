use crate::business::models::stream::LiveStreamStart;
use crate::persistence::entities::stream::{LiveStream, LiveStreamStatus};

impl From<&LiveStreamStart> for LiveStream {
    fn from(value: &LiveStreamStart) -> Self {
        Self {
            id: -1,
            video_id: value.video_id,
            start_time: chrono::Local::now(),
            status: LiveStreamStatus::Running,
        }
    }
}
