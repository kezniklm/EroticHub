use actix_session::Session;
use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::membership::details::template::MembershipDetailsTemplate;
use crate::api::templates::membership::payment_method::template::PaymentMethodTemplate;
use crate::api::templates::template::BaseTemplate;
use crate::business::facades::membership::{
    MembershipFacade, MembershipFacadeTrait, PaymentMethodInput,
};
use actix_web::{web, HttpResponse, Responder};
use askama_actix::TemplateToResponse;

// TODO: only allow for logged in user with the same user_id

pub async fn get_membership_details(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    user_id: web::Path<i32>,
) -> impl Responder {
    let membership_details = match membership_facade.get_membership_details(*user_id).await {
        Ok(membership_details) => membership_details,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let template = MembershipDetailsTemplate {
        user_id: *user_id,
        membership_url: match membership_details.payment_method.is_some() {
            true => format!("/membership/{}/payment", *user_id),
            false => format!("/membership/{}/payment-method", *user_id),
        },
        membership_details,
    };

    BaseTemplate::wrap(htmx_request, session, template).to_response()
}

pub async fn get_payment_method_form(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    user_id: web::Path<i32>,
) -> impl Responder {
    let has_payment_method = match membership_facade.has_payment_method(*user_id).await {
        Ok(has_payment_method) => has_payment_method,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let template = PaymentMethodTemplate {
        user_id: *user_id,
        has_payment_method,
    };

    BaseTemplate::wrap(htmx_request, session, template).to_response()
}

pub async fn change_payment_method(
    membership_facade: web::Data<MembershipFacade>,
    user_id: web::Path<i32>,
    payment_method_input: web::Form<PaymentMethodInput>,
) -> impl Responder {
    match membership_facade
        .change_payment_method(*user_id, payment_method_input.into_inner())
        .await
    {
        // TODO: if coming from "Get membership", go to .../payment
        Ok(_) => HttpResponse::SeeOther()
            .insert_header(("Location", format!("/membership/{}", *user_id)))
            .finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
