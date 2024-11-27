use crate::business::models::video::{ThumbnailUploadForm, VideoUploadForm};
use actix_multipart::form::MultipartForm;
use actix_web::{post, web, HttpResponse, Responder, Scope};

pub fn register_scope() -> Scope {
    web::scope("/video")
        .service(post_temp_video)
        .service(post_temp_thumbnail)
}

#[post("/temp/video")]
pub async fn post_temp_video(
    MultipartForm(form): MultipartForm<VideoUploadForm>,
) -> impl Responder {
    println!("{}", form.file.file_name.unwrap());
    println!("{}", form.file.content_type.unwrap());
    HttpResponse::Ok()
}

#[post("/temp/thumbnail")]
pub async fn post_temp_thumbnail(
    MultipartForm(form): MultipartForm<ThumbnailUploadForm>,
) -> impl Responder {
    println!("{}", form.file.file_name.unwrap());
    println!("{}", form.file.content_type.unwrap());
    HttpResponse::Ok()
}
