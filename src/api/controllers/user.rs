use crate::api::templates::user::list::template::UserListTemplate;
use crate::api::templates::user::register::template::UserRegisterTemplate;
use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use crate::business::models::user_register::UserRegister;
use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use askama::Template;
use log::error;

pub async fn register_form() -> impl Responder {
    let template = UserRegisterTemplate {};

    match template.render() {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn register_user(
    user_facade: web::Data<UserFacade>,
    user_register: web::Json<UserRegister>,
) -> impl Responder {
    match user_facade.register(user_register.into_inner()).await {
        Ok(_) => HttpResponse::Created(),
        Err(err) => {
            error!("Failed to register user: {:?}", err);
            HttpResponse::BadRequest()
        }
    }
}

pub async fn login(request: HttpRequest) -> impl Responder {
    // Some kind of authentication should happen here -
    // e.g. password-based, biometric, etc.
    // [...]

    // Attached a verified user identity to the active
    // session.
    Identity::login(&request.extensions(), "User1".into()).unwrap();

    HttpResponse::Ok()
}

pub async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::NoContent()
}

pub async fn list_users(user_facade: web::Data<UserFacade>) -> impl Responder {
    let users = match user_facade.list_users().await {
        Ok(users) => users,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let template = UserListTemplate { users };

    match template.render() {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
