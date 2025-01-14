use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::membership::deal::template::DealTemplate;
use crate::api::templates::membership::details::template::MembershipDetailsTemplate;
use crate::api::templates::membership::payment_method::template::PaymentMethodTemplate;
use crate::api::templates::template::BaseTemplate;
use crate::business::facades::membership::{
    MembershipFacade, MembershipFacadeTrait, PaymentMethodInput,
};
use actix_web::{web, HttpResponse, Responder};
use askama_actix::TemplateToResponse;
use config::Map;

// TODO: only allow for logged in user with the same user_id
// TODO: allow changing prices for admins

pub async fn get_membership_details(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    user_id: web::Path<i32>,
) -> impl Responder {
    let membership_details = match membership_facade.get_membership_details(*user_id).await {
        Ok(membership_details) => membership_details,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let template = MembershipDetailsTemplate {
        user_id: *user_id,
        membership_details,
    };

    BaseTemplate::wrap(htmx_request, template).to_response()
}

pub async fn get_payment_method_form(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    user_id: web::Path<i32>,
    query: web::Query<Map<String, String>>,
) -> impl Responder {
    let has_payment_method = match membership_facade.has_payment_method(*user_id).await {
        Ok(has_payment_method) => has_payment_method,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let template = PaymentMethodTemplate {
        user_id: *user_id,
        has_payment_method,
        back_to: query.get("back_to").unwrap_or(&"/".to_string()).to_string(),
    };

    BaseTemplate::wrap(htmx_request, template).to_response()
}

pub async fn change_payment_method(
    membership_facade: web::Data<MembershipFacade>,
    user_id: web::Path<i32>,
    payment_method_input: web::Form<PaymentMethodInput>,
) -> impl Responder {
    let back_to = payment_method_input.back_to.clone();
    match membership_facade
        .change_payment_method(*user_id, payment_method_input.into_inner())
        .await
    {
        Ok(_) => HttpResponse::SeeOther()
            .insert_header(("Location", back_to))
            .finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_deal_form(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    user_id: web::Path<i32>,
) -> impl Responder {
    let membership_details = match membership_facade.get_membership_details(*user_id).await {
        Ok(membership_details) => membership_details,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let template = DealTemplate {
        user_id: *user_id,
        membership_details,
    };

    BaseTemplate::wrap(htmx_request, template).to_response()
}

pub async fn get_payment_form(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    params: web::Path<(i32, i32)>,
) -> impl Responder {
    let user_id = params.0;
    let deal_id = params.1;

    // TODO:
    return HttpResponse::Ok().finish();
}
