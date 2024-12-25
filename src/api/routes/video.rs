use crate::api::controllers::video::{get_video, list_videos, save_video, upload_video_template};
use actix_web::web;

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/").route("", web::get().to(list_videos)))
        .service(
            web::scope("/video")
                .route("", web::get().to(get_video))
                .route("", web::post().to(save_video))
                .route("/new", web::get().to(upload_video_template)),
        );
}
