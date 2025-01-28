use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::business::models::user::UserRole;
use crate::business::models::user::UserSessionData;
use actix_session::Session;
use askama::Template;

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<T: Template> {
    child_template: T,
    htmx_request: HtmxRequest,
    user_session_data: Option<UserSessionData>,
    is_admin: bool,
    is_artist: bool,
}

impl<T: Template> BaseTemplate<T> {
    pub fn new(
        child_template: T,
        htmx_request: HtmxRequest,
        user_session_data: Option<UserSessionData>,
    ) -> Self {
        Self {
            child_template,
            htmx_request,
            user_session_data: user_session_data.clone(),
            is_admin: user_session_data.clone().as_ref().map_or(false, |session| {
                session.user_permissions.contains(&UserRole::Admin)
            }),
            is_artist: user_session_data.as_ref().map_or(false, |session| {
                session.user_permissions.contains(&UserRole::Artist)
            }),
        }
    }

    pub fn wrap(htmx_request: HtmxRequest, session: Session, child_template: T) -> Self {
        Self::new(
            child_template,
            htmx_request,
            session
                .get::<UserSessionData>("user_session_data")
                .unwrap_or(None),
        )
    }
}
