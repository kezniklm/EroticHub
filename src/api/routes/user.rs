use crate::api::controllers::user::list_users;
use actix_web::web;

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user").route("", web::get().to(list_users)), // .route("/{id}", web::get().to(get_user)) //example
                                                                  // .route("", web::post().to(create_user)),
    );
}
