use crate::api::controllers::stream::{start_stream, stop_stream, watch_stream};
use actix_web::web;

pub fn stream_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/stream")
            .route("/start", web::post().to(start_stream))
            .route("/{stream_id}/watch", web::get().to(watch_stream))
            .route("/{stream_id}/stop", web::delete().to(stop_stream)),
    );
}
