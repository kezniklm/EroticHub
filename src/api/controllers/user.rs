use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::user::list::template::UserListTemplate;
use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use actix_web::{web, HttpResponse, Responder};
use askama::Template;

pub async fn list_users(
    user_facade: web::Data<UserFacade>,
    htmx_request: HtmxRequest,
) -> impl Responder {
    let users = match user_facade.list_users().await {
        Ok(users) => users,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let template = UserListTemplate {
        users,
        htmx_request,
    };

    match template.render() {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
