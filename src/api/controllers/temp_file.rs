use crate::api::extractors::permissions_extractor::AsInteger;
use crate::api::extractors::template_extractor::TemplateReq;
use crate::api::templates::video::show::template::PlayerTemplate;
use crate::api::templates::video::upload::template::{
    ThumbnailPreviewTemplate, ThumbnailUploadInputTemplate, VideoPreviewTemplate,
    VideoUploadInputTemplate,
};
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::models::temp_file::{GetFileInputTemplate, TempFileInput};
use crate::business::models::user::UserRole::{self, Artist};
use crate::business::models::video::{ThumbnailUploadForm, VideoUploadForm};
use crate::business::Result;
use crate::configuration::models::Configuration;
use actix_files::NamedFile;
use actix_identity::Identity;
use actix_multipart::form::MultipartForm;
use actix_web::web::{Data, Path, Query};
use actix_web::{HttpResponse, Responder};
use actix_web_grants::protect;
use askama::Template;
use askama_actix::TemplateToResponse;

/// Creates new temporary video file
///
/// `POST /temp/video`
///
/// # Returns
/// `VideoPreviewTemplate` - includes video player together with hidden input including temp_file_id
#[protect(any("Artist"), ty = "UserRole")]
pub async fn post_temp_video(
    MultipartForm(form): MultipartForm<VideoUploadForm>,
    temp_file_facade: Data<TempFileFacade>,
    template_req: TemplateReq,
    config: Data<Configuration>,
    identity: Identity,
) -> Result<HttpResponse> {
    let file_name = form.file.file_name.clone().unwrap_or(String::new());
    let allowed_mime_types = config.app.video.accepted_mime_type.clone();

    temp_file_facade
        .check_mime_type(&form.file.file, allowed_mime_types)
        .await?;

    let temp_file_id = temp_file_facade
        .persist_temp_file(form.file.file, file_name, identity.id_i32()?)
        .await?;

    if !template_req.return_template {
        return Ok(HttpResponse::Created().body(temp_file_id.to_string()));
    }

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
#[protect(any("Artist"), ty = "UserRole")]
pub async fn post_temp_thumbnail(
    MultipartForm(form): MultipartForm<ThumbnailUploadForm>,
    temp_file_facade: Data<TempFileFacade>,
    template_req: TemplateReq,
    config: Data<Configuration>,
    identity: Identity,
) -> Result<impl Responder> {
    let file_name = form.file.file_name.clone().unwrap_or(String::new());
    let allowed_mime_types = config.app.thumbnail.accepted_mime_type.clone();

    temp_file_facade
        .check_mime_type(&form.file.file, allowed_mime_types)
        .await?;

    let temp_file_id = temp_file_facade
        .persist_temp_file(form.file.file, file_name, identity.id_i32()?)
        .await?;

    if !template_req.return_template {
        return Ok(HttpResponse::Created().body(temp_file_id.to_string()));
    }

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
#[protect(any("Artist"), ty = "UserRole")]
pub async fn get_temp_file(
    temp_file: Path<i32>,
    temp_file_facade: Data<TempFileFacade>,
    identity: Identity,
) -> Result<NamedFile> {
    let file = temp_file_facade
        .get_temp_file(temp_file.into_inner(), identity.id_i32()?)
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
#[protect(any("Artist"), ty = "UserRole")]
pub async fn delete_temp_file(
    temp_file: Path<i32>,
    Query(temp_file_type): Query<GetFileInputTemplate>,
    temp_file_facade: Data<TempFileFacade>,
    config: Data<Configuration>,
    identity: Identity,
) -> Result<impl Responder> {
    temp_file_facade
        .delete_temp_file(temp_file.into_inner(), identity.id_i32()?)
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
#[protect(any("Artist"), ty = "UserRole")]
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
