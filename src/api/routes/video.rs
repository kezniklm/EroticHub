use crate::api::controllers::video::{list_videos, upload_video_template};
use actix_web::web;

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/").route("", web::get().to(list_videos)))
        .service(web::scope("/video").route("/new", web::get().to(upload_video_template)));
}
