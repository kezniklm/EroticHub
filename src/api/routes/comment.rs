use crate::api::controllers::comment::{create_comment, get_comments_to_video};
use actix_web::web;

pub fn comment_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comments")
            .route("/{id}", web::get().to(get_comments_to_video))
            .route("", web::post().to(create_comment)),
    );
}
