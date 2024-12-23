use crate::api::extractors::htmx_extractor::HtmxRequest;
use askama::Template;

#[derive(Template)]
#[template(path = "video/list/index.html")]
pub struct VideoListTemplate {
    pub htmx_request: HtmxRequest,
}
