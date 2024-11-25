use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use actix_web::{get, web, HttpResponse, Responder};

#[get("/")]
pub async fn list_users(user_facade: web::Data<UserFacade>) -> impl Responder {
    match user_facade.list_users().await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
