use crate::api::controllers::stream::start_stream;
use actix_web::web;

pub fn stream_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/stream").route("/start", web::post().to(start_stream)));
}
