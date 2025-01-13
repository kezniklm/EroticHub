use crate::api::controllers::video::{
    create_video, delete_video, edit_video_template, get_thumbnail, get_video, list_videos,
    patch_video, upload_video_template, watch_video,
};
use actix_web::web;

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/").route("", web::get().to(list_videos)))
        .service(
            web::scope("/video")
                .route("/new", web::get().to(upload_video_template))
                .route("{id}", web::get().to(get_video))
                .route("{id}", web::patch().to(patch_video))
                .route("{id}", web::delete().to(delete_video))
                .route("/{id}/edit", web::get().to(edit_video_template))
                .route("/{id}/watch", web::get().to(watch_video))
                .route("", web::post().to(create_video)),
        )
        .service(web::scope("/thumbnail").route("/{id}", web::get().to(get_thumbnail)));
}
