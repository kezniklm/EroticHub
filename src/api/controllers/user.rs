use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::template::BaseTemplate;
use crate::api::templates::user::list::template::UserListTemplate;
use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use actix_web::{web, HttpResponse, Responder};
use askama_actix::TemplateToResponse;

pub async fn list_users(
    user_facade: web::Data<UserFacade>,
    htmx_request: HtmxRequest,
) -> impl Responder {
    let users = match user_facade.list_users().await {
        Ok(users) => users,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let template = UserListTemplate { users };

    BaseTemplate::wrap(htmx_request, template).to_response()
}
