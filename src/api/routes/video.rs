use crate::api::controllers::video::list_videos;
use actix_web::web;

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/").route("", web::get().to(list_videos)));
}
