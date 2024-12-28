use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::template::BaseTemplate;
use crate::api::templates::video::list::template::VideoListTemplate;
use crate::api::templates::video::upload::template::VideoUploadTemplate;
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
use actix_web::web::{Data, Query};
use actix_web::{get, post, web, HttpResponse, Responder, Result, Scope};
use tempfile::NamedTempFile;

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
) -> Result<impl Responder> {
    let video = video_facade.save_video(1, form).await?;

    Ok(HttpResponse::Created().json(video))
}

#[get("")]
pub async fn get_video(
    Query(request): Query<PlayableVideoReq>,
    video_facade: Data<VideoFacade>,
) -> Result<NamedFile> {
    let video = video_facade.get_playable_video(request.id, 1).await?;

    Ok(video)
}

#[post("/video")]
pub async fn post_temp_video(
    MultipartForm(form): MultipartForm<VideoUploadForm>,
    temp_file_facade: Data<TempFileFacade>,
    config: Data<Configuration>,
) -> Result<impl Responder> {
    let file_name = form.file.file_name.clone().unwrap_or(String::new());
    let allowed_mime_types = config.app.video.accepted_mime_type.clone();
    // TODO: permissions - check if user can upload videos

    let content_type = get_content_type_string(&form.file);
    temp_file_facade
        .check_mime_type(content_type, allowed_mime_types)
        .await?;
    upload_temp_file(temp_file_facade, form.file.file, file_name).await
}

#[post("/thumbnail")]
pub async fn post_temp_thumbnail(
    MultipartForm(form): MultipartForm<ThumbnailUploadForm>,
    temp_file_facade: Data<TempFileFacade>,
    config: Data<Configuration>,
) -> Result<impl Responder> {
    let file_name = form.file.file_name.clone().unwrap_or(String::new());
    let allowed_mime_types = config.app.thumbnail.accepted_mime_type.clone();
    // TODO: permissions - check if user can upload videos

    let content_type = get_content_type_string(&form.file);
    temp_file_facade
        .check_mime_type(content_type, allowed_mime_types)
        .await?;
    Ok(upload_temp_file(temp_file_facade, form.file.file, file_name).await)
}

async fn upload_temp_file(
    temp_file_facade: Data<TempFileFacade>,
    file: NamedTempFile,
    file_name: String,
) -> Result<impl Responder> {
    let temp_file_res = temp_file_facade
        .persist_temp_file(file, file_name, 1)
        .await?;
    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .json(temp_file_res))
}

fn get_content_type_string(temp_file: &TempFile) -> Option<String> {
    temp_file
        .content_type
        .clone()
        .map(|content_type| content_type.to_string())
}

pub async fn list_videos(htmx_request: HtmxRequest) -> impl Responder {
    BaseTemplate::wrap(htmx_request, VideoListTemplate {})
}

pub async fn upload_video_template(htmx_request: HtmxRequest) -> impl Responder {
    BaseTemplate::wrap(htmx_request, VideoUploadTemplate {})
}
