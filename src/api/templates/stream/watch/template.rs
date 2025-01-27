use crate::business::models::stream::LiveStream;
#[allow(unused_imports)] // Used in stream.html template
use crate::business::models::stream::LiveStreamStatus;
use crate::business::models::video::Video;
use askama_actix::Template;

#[derive(Template)]
#[template(path = "stream/watch/stream.html")]
pub struct WatchStreamTemplate {
    pub stream: LiveStream,
    pub video: Video,
    pub is_owner: bool,
}
