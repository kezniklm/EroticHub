use crate::business::models::video::{ThumbnailUploadForm, VideoUploadForm};
use actix_multipart::form::MultipartForm;
use actix_web::{post, web, HttpResponse, Responder, Scope};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use tempfile::NamedTempFile;
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};

pub fn register_scope() -> Scope {
    let temp_scope = web::scope("/temp").service(post_temp_video).service(post_temp_video);
    web::scope("/video")
        .service(temp_scope)
}

#[post("/video")]
pub async fn post_temp_video(
    MultipartForm(form): MultipartForm<VideoUploadForm>,
    temp_file_facade: Data<TempFileFacade>
) -> impl Responder {
    let file_name = form.file.file_name.unwrap_or(String::new());

    // TODO: permissions - check if user can upload videos
    upload_temp_file(temp_file_facade, form.file.file, file_name).await
}

#[post("/thumbnail")]
pub async fn post_temp_thumbnail(
    MultipartForm(form): MultipartForm<ThumbnailUploadForm>,
    temp_file_facade: Data<TempFileFacade>
) -> impl Responder {
    let file_name = form.file.file_name.unwrap_or(String::new());

    // TODO: permissions - check if user can upload videos
    upload_temp_file(temp_file_facade, form.file.file, file_name).await
}

async fn upload_temp_file(temp_file_facade: Data<TempFileFacade>, file: NamedTempFile, file_name: String) -> impl Responder {
    match temp_file_facade.persist_temp_file(file, file_name, 1).await {
        Ok(temp_file_res) => {
            HttpResponse::Ok().json(temp_file_res)
        }
        Err(err) => {
            log::error!("Failed to create temp file: {:#?}", err);
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("Failed to create temp file")
        }
    }
}
