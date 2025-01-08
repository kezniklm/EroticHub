use crate::api::controllers::user::{login, login_form, logout, register_form, register_user};
use actix_web::web;

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("/register", web::get().to(register_form))
            .route("/register", web::post().to(register_user))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout)),
    );
}
