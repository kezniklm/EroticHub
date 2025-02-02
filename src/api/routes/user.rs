use crate::api::controllers::user::{
    change_password, change_password_form, delete, delete_form, like_video, liked_videos,
    likes_page, login, login_form, logout, profile_picture_update, register_form, register_user,
    user_detail, user_update, validate_email, validate_username,
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
            .route("/account", web::get().to(user_detail))
            .route("/account/edit", web::post().to(user_update))
            .route(
                "/account/profile-picture-update",
                web::post().to(profile_picture_update),
            )
            .route("/likes", web::get().to(likes_page))
            .route("/liked-videos", web::get().to(liked_videos))
            .route("/like/{video_id}", web::post().to(like_video))
            .route("/change-password", web::get().to(change_password_form))
            .route("/change-password", web::post().to(change_password))
            .route("/delete", web::get().to(delete_form))
            .route("/delete", web::post().to(delete)),
    );
}
