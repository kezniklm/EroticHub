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
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{Error, HttpResponse, Responder};
use askama_actix::TemplateToResponse;
use log::error;

/// Creates new temporary video file
///
/// `POST /temp/video`
///
/// # Returns
/// `VideoPreviewTemplate` - includes video player together with hidden input including temp_file_id
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

/// Creates new temporary file for thumbnail
///
/// `POST /temp/thumbnail`
///
/// # Returns
/// `ThumbnailPreviewTemplate` - Thumbnail preview template together with hidden input with temp_file_id
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

/// Get a temporary file for a preview
///
/// `GET /temp/{temp_file}`
///
/// # Returns
/// Temporary file
pub async fn get_temp_file(
    temp_file: Path<i32>,
    temp_file_facade: Data<TempFileFacade>,
) -> actix_web::Result<NamedFile> {
    // TODO: CHECK PERMISSIONS
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

/// Deletes temporary file
///
/// `DELETE /temp/{temp_file}`
///
/// # Query params
/// - `GetFileInputTemplate` - decides which input template should be returned
///
/// # Returns
/// Upload input template based on the request, so user can upload the item again
pub async fn delete_temp_file(
    temp_file: Path<i32>,
    Query(temp_file_type): Query<GetFileInputTemplate>,
    temp_file_facade: Data<TempFileFacade>,
    config: Data<Configuration>,
) -> impl Responder {
    // TODO: Check permissions
    match temp_file_facade
        .delete_temp_file(temp_file.into_inner(), 1)
        .await
    {
        Ok(_) => Ok(get_upload_template(temp_file_type, config)),
        Err(err) => {
            error!("Error while removing temporary video: {:#?}", err);
            Err(Error::from(actix_web::error::InternalError::new(
                "Deletion of temporary video file failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}

/// Returns templates for file inputs. Can be used in case when user want's to re-upload the video,
/// but the video is no longer temporary file (video is already saved).
///
/// `GET /temp/template`
///
/// # Body params
/// - `Json<GetFileInputTemplate>` - decides which input template should be returned
///
/// # Returns
/// Template HTML based on the request.
pub async fn get_input_template(
    Json(temp_file_type): Json<GetFileInputTemplate>,
    config: Data<Configuration>,
) -> impl Responder {
    get_upload_template(temp_file_type, config)
}

fn get_upload_template(
    temp_file_type: GetFileInputTemplate,
    config: Data<Configuration>,
) -> HttpResponse {
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
