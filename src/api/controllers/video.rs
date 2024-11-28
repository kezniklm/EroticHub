use crate::business::models::video::{ThumbnailUploadForm, VideoUploadForm};
use actix_multipart::form::MultipartForm;
use actix_web::{post, web, HttpResponse, Responder, Scope};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use log::error;
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};

pub fn register_scope() -> Scope {
    web::scope("/video")
        .service(post_temp_video)
        .service(post_temp_thumbnail)
}

#[post("/temp/video")]
pub async fn post_temp_video(
    MultipartForm(form): MultipartForm<VideoUploadForm>,
    temp_file_facade: Data<TempFileFacade>
) -> HttpResponse {
    let file_name = form.file.file_name.unwrap_or(String::new());

    match temp_file_facade.persist_temp_file(form.file.file, file_name, 1).await {
        Ok(_) => {
            HttpResponse::from(HttpResponse::Ok())
        }
        Err(err) => {
            log::error!("Failed to create temp video file: {}", err);
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("Failed to create temp video file")
        }
    }
}

#[post("/temp/thumbnail")]
pub async fn post_temp_thumbnail(
    MultipartForm(form): MultipartForm<ThumbnailUploadForm>,
) -> impl Responder {
    println!("{}", form.file.file_name.unwrap());
    println!("{}", form.file.content_type.unwrap());
    HttpResponse::Ok()
}
