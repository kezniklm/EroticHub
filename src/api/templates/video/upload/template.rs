use crate::api::extractors::htmx_extractor::HtmxRequest;
use askama_actix::Template;

#[derive(Template)]
#[template(path = "video/upload/index.html")]
pub struct VideoUploadTemplate {
    pub htmx_request: HtmxRequest,
}
