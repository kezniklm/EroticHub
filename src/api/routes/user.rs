use crate::api::controllers::user::{
    detail, liked_videos, login, login_form, logout, register_form, register_user, validate_email,
    validate_username,
};
use actix_web::web;
use actix_web::web::scope;

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        scope("/user/validate")
            .route("/username", web::get().to(validate_username))
            .route("/email", web::get().to(validate_email)),
    );
    cfg.service(
        scope("/user")
            .route("/register", web::get().to(register_form))
            .route("/register", web::post().to(register_user))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/logout", web::get().to(logout))
            .route("/account", web::get().to(detail))
            .route("/liked-videos", web::get().to(liked_videos)),
    );
}
