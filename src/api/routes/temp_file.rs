use crate::api::controllers::temp_file::{
    delete_temp_file, get_input_template, get_temp_file, post_temp_thumbnail, post_temp_video,
};
use actix_web::web;

pub fn temp_file_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/temp")
            .route("/{temp_file}", web::get().to(get_temp_file))
            .route("/video", web::post().to(post_temp_video))
            .route("/{temp_file}", web::delete().to(delete_temp_file))
            .route("/thumbnail", web::post().to(post_temp_thumbnail))
            .route("/template", web::get().to(get_input_template)),
    );
}
