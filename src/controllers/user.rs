use actix_web::{get, web, HttpResponse, Responder};

use crate::repositories::user::{PostgresUserRepo, UserRepo};

#[get("/")]
pub async fn list_users(repo: web::Data<PostgresUserRepo>) -> impl Responder {
    match repo.list_users().await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
