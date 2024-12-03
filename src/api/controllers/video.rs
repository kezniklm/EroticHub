use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::facades::video::{VideoFacade, VideoFacadeTrait};
use crate::business::models::video::{
    PlayableVideoReq, ThumbnailUploadForm, VideoUploadData, VideoUploadForm,
};
use crate::configuration::models::Configuration;
use actix_files::NamedFile;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Query};
use actix_web::{get, post, web, Error, HttpResponse, Responder, Result, Scope, HttpResponse};
use log::error;
use tempfile::NamedTempFile;
use crate::api::templates::video::list::template::VideoListTemplate;
use askama::Template;

pub fn register_scope() -> Scope {
    let temp_scope = web::scope("/temp")
        .service(post_temp_video)
        .service(post_temp_thumbnail);
    web::scope("/video")
        .service(temp_scope)
        .service(save_video)
        .service(get_video)
}

#[post("/")]
pub async fn save_video(
    web::Form(form): web::Form<VideoUploadData>,
    video_facade: Data<VideoFacade>,
) -> impl Responder {
    match video_facade.save_video(1, form).await {
        Ok(video) => HttpResponse::Ok().json(video),
        Err(err) => {
            error!("Error while saving video: {:#?}", err);
            HttpResponse::InternalServerError()
                .content_type(ContentType::plaintext())
                .finish()
        }
    }
}

#[get("")]
pub async fn get_video(
    Query(request): Query<PlayableVideoReq>,
    video_facade: Data<VideoFacade>,
) -> Result<NamedFile> {
    match video_facade.get_playable_video(request.id, 1).await {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error while returning playable video: {:#?}", err);
            Err(Error::from(actix_web::error::InternalError::new(
                "Loading of video file failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}

#[post("/video")]
pub async fn post_temp_video(
    MultipartForm(form): MultipartForm<VideoUploadForm>,
    temp_file_facade: Data<TempFileFacade>,
    config: Data<Configuration>,
) -> HttpResponse {
    let file_name = form.file.file_name.clone().unwrap_or(String::new());
    let allowed_mime_types = config.app.video.accepted_mime_type.clone();
    // TODO: permissions - check if user can upload videos

    let content_type = get_content_type_string(&form.file);
    if temp_file_facade
        .check_mime_type(content_type, allowed_mime_types)
        .await
        .is_err()
    {
        return HttpResponse::build(StatusCode::UNSUPPORTED_MEDIA_TYPE).finish();
    };
    upload_temp_file(temp_file_facade, form.file.file, file_name).await
}

#[post("/thumbnail")]
pub async fn post_temp_thumbnail(
    MultipartForm(form): MultipartForm<ThumbnailUploadForm>,
    temp_file_facade: Data<TempFileFacade>,
    config: Data<Configuration>,
) -> impl Responder {
    let file_name = form.file.file_name.clone().unwrap_or(String::new());
    let allowed_mime_types = config.app.thumbnail.accepted_mime_type.clone();
    // TODO: permissions - check if user can upload videos

    let content_type = get_content_type_string(&form.file);
    if temp_file_facade
        .check_mime_type(content_type, allowed_mime_types)
        .await
        .is_err()
    {
        return HttpResponse::build(StatusCode::UNSUPPORTED_MEDIA_TYPE).finish();
    };
    upload_temp_file(temp_file_facade, form.file.file, file_name).await
}

async fn upload_temp_file(
    temp_file_facade: Data<TempFileFacade>,
    file: NamedTempFile,
    file_name: String,
) -> HttpResponse {
    match temp_file_facade.persist_temp_file(file, file_name, 1).await {
        Ok(temp_file_res) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(temp_file_res),
        Err(err) => {
            log::error!("Failed to create temp file: {:#?}", err);
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to create temp file")
        }
    }
}

fn get_content_type_string(temp_file: &TempFile) -> Option<String> {
    temp_file
        .content_type
        .clone()
        .map(|content_type| content_type.to_string())
}

pub async fn list_videos() -> impl Responder {
    let template = VideoListTemplate {};

    match template.render() {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
