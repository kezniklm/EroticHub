use chrono::Local;

pub struct LiveStream {
    id: u64,
    video_id: u64,
    start_time: chrono::DateTime<Local>,
}
