use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::admin::deals::template::AdminDealsTemplate;
use crate::api::templates::admin::index::template::AdminIndexTemplate;
use crate::api::templates::admin::template::AdminSectionTemplate;
use crate::api::templates::template::BaseTemplate;
use crate::business::facades::membership::{MembershipFacade, MembershipFacadeTrait};
use crate::business::models::user::UserRole::{self, Admin};

use actix_session::Session;
use actix_web::{web, Responder, Result};
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

#[protect(any("Admin"), ty = "UserRole")]
pub async fn get_admin_deals(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
) -> Result<impl Responder> {
    let deals = membership_facade.get_deals().await?;

    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        AdminSectionTemplate::wrap(AdminDealsTemplate { deals }),
    ))
}
