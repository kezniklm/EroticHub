use super::utils::route_util::add_redirect_header;

use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::extractors::permissions_extractor::AsInteger;
use crate::api::templates::membership::deal::template::DealTemplate;
use crate::api::templates::membership::details::template::MembershipDetailsTemplate;
use crate::api::templates::membership::payment::template::PaymentTemplate;
use crate::api::templates::membership::payment_method::template::PaymentMethodTemplate;
use crate::api::templates::template::BaseTemplate;
use crate::business::facades::membership::{
    MembershipFacade, MembershipFacadeTrait, PaymentMethodInput,
};
use crate::business::models::error::AppError;
use crate::business::models::user::UserRole::{self, Registered};

use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder, Result};
use actix_web_grants::protect;
use config::Map;

// TODO: allow changing prices for admins

#[protect(any("Registered"), ty = "UserRole")]
pub async fn get_membership_details(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    identity: Identity,
) -> Result<impl Responder> {
    let user_id = identity.id_i32()?;

    let membership_details = membership_facade.get_membership_details(user_id).await?;

    let template = MembershipDetailsTemplate { membership_details };

    Ok(BaseTemplate::wrap(htmx_request, session, template))
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn get_payment_method_form(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    query: web::Query<Map<String, String>>,
    identity: Identity,
) -> Result<impl Responder> {
    let user_id = identity.id_i32()?;

    let has_payment_method = membership_facade.has_payment_method(user_id).await?;

    let template = PaymentMethodTemplate {
        has_payment_method,
        back_to: query.get("back_to").unwrap_or(&"/".to_string()).to_string(),
    };

    Ok(BaseTemplate::wrap(htmx_request, session, template))
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn change_payment_method(
    membership_facade: web::Data<MembershipFacade>,
    payment_method_input: web::Form<PaymentMethodInput>,
    identity: Identity,
) -> Result<impl Responder> {
    let user_id = identity.id_i32()?;
    let back_to = payment_method_input.back_to.clone();

    membership_facade
        .change_payment_method(user_id, payment_method_input.into_inner())
        .await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header(&back_to, &mut response)?;
    Ok(response)
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn get_deal_form(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    identity: Identity,
) -> Result<impl Responder> {
    let user_id = identity.id_i32()?;

    let membership_details = membership_facade.get_membership_details(user_id).await?;

    let deals = membership_facade.get_deals().await?;

    let template = DealTemplate {
        membership_details,
        deals,
    };

    Ok(BaseTemplate::wrap(htmx_request, session, template))
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn get_payment_form(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    deal_id: web::Path<i32>,
    identity: Identity,
) -> Result<impl Responder> {
    let user_id = identity.id_i32()?;

    let membership_details = membership_facade.get_membership_details(user_id).await?;

    let deal = membership_facade.get_deal(*deal_id).await?.ok_or_else(|| {
        AppError::new(
            "Deal not found",
            crate::business::models::error::AppErrorKind::NotFound,
        )
    })?;

    let template = PaymentTemplate {
        membership_details,
        deal,
    };

    Ok(BaseTemplate::wrap(htmx_request, session, template))
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn pay(
    membership_facade: web::Data<MembershipFacade>,
    deal_id: web::Path<i32>,
    identity: Identity,
) -> Result<impl Responder> {
    let user_id = identity.id_i32()?;

    if !membership_facade.has_payment_method(user_id).await? {
        let mut response = HttpResponse::NoContent().finish();
        add_redirect_header(
            &format!(
                "/membership/payment-method?back_to=/membership/deal/{}",
                deal_id
            ),
            &mut response,
        )?;
        return Ok(response);
    }

    membership_facade.pay(user_id, *deal_id).await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/membership", &mut response)?;
    Ok(response)
}
