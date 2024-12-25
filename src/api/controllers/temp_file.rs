use crate::api::templates::video::upload::template::{
    ThumbnailPreviewTemplate, ThumbnailUploadInputTemplate, VideoPreviewTemplate,
    VideoUploadInputTemplate,
};
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::models::temp_file::{GetFileInputTemplate, TempFileInput};
use crate::business::models::video::{ThumbnailUploadForm, VideoUploadForm};
use crate::configuration::models::Configuration;
use actix_files::NamedFile;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{Error, HttpResponse, Responder};
use askama_actix::TemplateToResponse;
use log::error;

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

    match temp_file_facade
        .persist_temp_file(form.file.file, file_name, 1)
        .await
    {
        Ok(temp_file_id) => {
            let template = VideoPreviewTemplate {
                temp_file_id: Some(temp_file_id),
                file_path: format!("/temp/{temp_file_id}"),
            };

            template.to_response()
        }
        Err(err) => {
            log::error!("Failed to create temp file: {:#?}", err);
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to create temp file")
        }
    }
}

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
    match temp_file_facade
        .persist_temp_file(form.file.file, file_name, 1)
        .await
    {
        Ok(temp_file_id) => {
            let template = ThumbnailPreviewTemplate {
                temp_file_id: Some(temp_file_id),
                file_path: format!("/temp/{temp_file_id}"),
            };

            template.to_response()
        }
        Err(err) => {
            log::error!("Failed to create temp file: {:#?}", err);
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to create temp file")
        }
    }
}

pub async fn get_temp_file(
    temp_file: Path<i32>,
    temp_file_facade: Data<TempFileFacade>,
) -> actix_web::Result<NamedFile> {
    match temp_file_facade
        .get_temp_file(temp_file.into_inner(), 1)
        .await
    {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error while saving video: {:#?}", err);
            Err(Error::from(actix_web::error::InternalError::new(
                "Loading of video file failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}

pub async fn delete_temp_video(
    temp_file: Path<i32>,
    temp_file_facade: Data<TempFileFacade>,
    config: Data<Configuration>,
) -> impl Responder {
    match temp_file_facade
        .delete_temp_file(temp_file.into_inner(), 1)
        .await
    {
        Ok(_) => Ok(VideoUploadInputTemplate::new(config.into_inner()).to_response()),
        Err(err) => {
            error!("Error while removing temporary video: {:#?}", err);
            Err(Error::from(actix_web::error::InternalError::new(
                "Deletion of temporary video file failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}

pub async fn delete_temp_thumbnail(
    temp_file: Path<i32>,
    temp_file_facade: Data<TempFileFacade>,
    config: Data<Configuration>,
) -> impl Responder {
    match temp_file_facade
        .delete_temp_file(temp_file.into_inner(), 1)
        .await
    {
        Ok(_) => Ok(ThumbnailUploadInputTemplate::new(config.into_inner()).to_response()),
        Err(err) => {
            error!("Error while removing temporary video: {:#?}", err);
            Err(Error::from(actix_web::error::InternalError::new(
                "Deletion of temporary video file failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}

pub async fn get_input_template(
    Json(temp_file_type): Json<GetFileInputTemplate>,
    config: Data<Configuration>,
) -> impl Responder {
    match temp_file_type.input_type {
        TempFileInput::Video => VideoUploadInputTemplate::new(config.into_inner()).to_response(),
        TempFileInput::Thumbnail => {
            ThumbnailUploadInputTemplate::new(config.into_inner()).to_response()
        }
    }
}

fn get_content_type_string(temp_file: &TempFile) -> Option<String> {
    temp_file
        .content_type
        .clone()
        .map(|content_type| content_type.to_string())
}
