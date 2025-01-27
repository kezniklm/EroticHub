use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::extractors::permissions_extractor::IsRole;
use crate::business::models::user::UserSessionData;
use actix_session::Session;
use askama::Template;

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<T: Template> {
    child_template: T,
    htmx_request: HtmxRequest,
    user_session_data: Option<UserSessionData>,
    session: Session,
}

impl<T: Template> BaseTemplate<T> {
    pub fn wrap(htmx_request: HtmxRequest, session: Session, child_template: T) -> Self {
        Self {
            htmx_request,
            session: session.clone(),
            user_session_data: session
                .get::<UserSessionData>("user_session_data")
                .unwrap_or(None),
            child_template,
        }
    }
}
