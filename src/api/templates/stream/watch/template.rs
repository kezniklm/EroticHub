use crate::business::models::stream::LiveStream;
use askama_actix::Template;
#[allow(unused_imports)] // Used in stream.html template
use crate::business::models::stream::LiveStreamStatus;

#[derive(Template)]
#[template(path = "stream/watch/stream.html")]
pub struct WatchStreamTemplate {
    pub stream: LiveStream,
}
