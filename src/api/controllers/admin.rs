use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::admin::index::template::AdminIndexTemplate;
use crate::api::templates::admin::template::AdminSectionTemplate;
use crate::api::templates::template::BaseTemplate;
use crate::business::models::user::UserRole::{self, Admin};

use actix_session::Session;
use actix_web::{Responder, Result};
use actix_web_grants::protect;

// TODO: changing deals/prices of memberships
// TODO: changing categories

#[protect(any("Admin"), ty = "UserRole")]
pub async fn get_admin_section(
    htmx_request: HtmxRequest,
    session: Session,
) -> Result<impl Responder> {
    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        AdminSectionTemplate::wrap(AdminIndexTemplate {}),
    ))
}
