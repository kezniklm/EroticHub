use crate::api::templates::video::show::template::PlayerTemplate;
use crate::api::templates::video::upload::template::{
    ThumbnailPreviewTemplate, ThumbnailUploadInputTemplate, VideoPreviewTemplate,
    VideoUploadInputTemplate,
};
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::models::temp_file::{GetFileInputTemplate, TempFileInput};
use crate::business::models::video::{ThumbnailUploadForm, VideoUploadForm};
use crate::business::Result;
use crate::configuration::models::Configuration;
use actix_files::NamedFile;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use actix_web::web::{Data, Path, Query};
use actix_web::{HttpResponse, Responder};
use askama::Template;
use askama_actix::TemplateToResponse;

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
) -> Result<HttpResponse> {
    let file_name = form.file.file_name.clone().unwrap_or(String::new());
    let allowed_mime_types = config.app.video.accepted_mime_type.clone();
    // TODO: permissions - check if user can upload videos

    let content_type = get_content_type_string(&form.file);
    temp_file_facade
        .check_mime_type(content_type, allowed_mime_types)
        .await?;

    let temp_file_id = temp_file_facade
        .persist_temp_file(form.file.file, file_name, 1)
        .await?;

    let template = VideoPreviewTemplate {
        temp_file_id: Some(temp_file_id),
        player_template: PlayerTemplate::from_temporary(temp_file_id),
    };

    Ok(HttpResponse::Created().body(template.render().unwrap()))
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
) -> Result<impl Responder> {
    let file_name = form.file.file_name.clone().unwrap_or(String::new());
    let allowed_mime_types = config.app.thumbnail.accepted_mime_type.clone();
    // TODO: permissions - check if user can upload videos

    let content_type = get_content_type_string(&form.file);
    temp_file_facade
        .check_mime_type(content_type, allowed_mime_types)
        .await?;

    let temp_file_id = temp_file_facade
        .persist_temp_file(form.file.file, file_name, 1)
        .await?;

    let template = ThumbnailPreviewTemplate {
        temp_file_id: Some(temp_file_id),
        file_path: format!("/temp/{temp_file_id}"),
    };

    Ok(HttpResponse::Created().body(template.render().unwrap()))
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
) -> Result<NamedFile> {
    // TODO: CHECK PERMISSIONS
    let file = temp_file_facade
        .get_temp_file(temp_file.into_inner(), 1)
        .await?;

    Ok(file)
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
) -> Result<impl Responder> {
    // TODO: Check permissions
    temp_file_facade
        .delete_temp_file(temp_file.into_inner(), 1)
        .await?;

    Ok(get_upload_template(temp_file_type, config))
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
    Query(temp_file_type): Query<GetFileInputTemplate>,
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
