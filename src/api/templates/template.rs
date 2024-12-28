use crate::api::extractors::htmx_extractor::HtmxRequest;
use askama::Template;

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<T: Template> {
    child_template: T,
    htmx_request: HtmxRequest,
}

impl<T: Template> BaseTemplate<T> {
    pub fn wrap(htmx_request: HtmxRequest, child_template: T) -> Self {
        Self {
            htmx_request,
            child_template,
        }
    }
}
