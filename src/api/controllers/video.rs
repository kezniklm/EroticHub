use crate::api::controllers::utils::route_util::build_watch_path;
use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::template::BaseTemplate;
use crate::api::templates::video::edit::template::EditVideoTemplate;
use crate::api::templates::video::list::template::VideoListTemplate;
use crate::api::templates::video::show::template::{PlayerTemplate, ShowVideoTemplate};
use crate::api::templates::video::upload::template::{
    ThumbnailPreviewTemplate, ThumbnailUploadInputTemplate, VideoPreviewTemplate,
    VideoUploadInputTemplate, VideoUploadTemplate,
};
use crate::business::facades::video::{VideoFacade, VideoFacadeTrait};
use crate::business::models::video::{GetVideoByIdReq, VideoEditReq, VideoUploadReq};
use crate::configuration::models::Configuration;
use actix_files::NamedFile;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::web::{Data, Form, Path};
use actix_web::{HttpResponse, Responder, Result};
use askama_actix::TemplateToResponse;
use std::str::FromStr;

/// Creates new video
///
/// `POST /video`
///
/// # Form params
/// `VideoUploadReq` - data of uploaded video
///
/// # Returns
/// Redirects user to the newly created video
pub async fn create_video(
    Form(form): Form<VideoUploadReq>,
    video_facade: Data<VideoFacade>,
) -> Result<impl Responder> {
    let video = video_facade.save_video(1, form).await?;
    let video_id = video.id;

    let mut response = HttpResponse::Created().finish();

    add_redirect_header(build_watch_path(video_id).as_str(), &mut response)?;

    Ok(response)
}

/// Patches (updates) video
///
/// `PATCH /video/{id}`
///
/// # Form params
/// `VideoEditReq` - data of uploaded video
///
/// # Returns
/// Redirects user to the patched video
pub async fn patch_video(
    path: Path<GetVideoByIdReq>,
    Form(form): Form<VideoEditReq>,
    video_facade: Data<VideoFacade>,
) -> Result<impl Responder> {
    let video = video_facade.patch_video(1, path.id, form).await?;

    let template = ShowVideoTemplate {
        video,
        player_template: PlayerTemplate::from_saved(path.id),
    };
    let mut response = template.to_response();

    add_redirect_header(build_watch_path(path.id).as_str(), &mut response)?;

    Ok(response)
}

/// Deletes video
///
/// `DELETE /video/{id}`
///
/// # Returns
/// Redirects user to the main page
pub async fn delete_video(
    path: Path<GetVideoByIdReq>,
    video_facade: Data<VideoFacade>,
) -> Result<impl Responder> {
    video_facade.delete_video(1, path.id).await?;

    let mut response = HttpResponse::NoContent().finish();

    add_redirect_header("/", &mut response)?;
    Ok(response)
}

/// Returns video file
///
/// `GET /video/{id}`
///
/// # Returns
/// File with the video
pub async fn get_video(
    request: Path<GetVideoByIdReq>,
    video_facade: Data<VideoFacade>,
) -> Result<NamedFile> {
    let file = video_facade.get_playable_video(request.id, 1).await?;
    Ok(file)
}

/// Returns thumbnail file
///
/// `GET /thumbnail/{id}`
///
/// # Returns
/// File with the thumbnail
pub async fn get_thumbnail(
    request: Path<GetVideoByIdReq>,
    video_facade: Data<VideoFacade>,
) -> Result<NamedFile> {
    let file = video_facade.get_thumbnail_file(request.id, 1).await?;
    Ok(file)
}

/// Returns template which displays the video
/// TODO: Display some placeholder when user doesn't have permissions to see the video!
///
/// `GET /{id}/watch`
///
/// # Returns
/// `ShowVideoTemplate` - template with video view
pub async fn watch_video(
    req: Path<GetVideoByIdReq>,
    video_facade: Data<VideoFacade>,
    htmx_request: HtmxRequest,
) -> Result<impl Responder> {
    let video = video_facade.get_video_model(req.id, 1).await?;
    let video_id = video.id;
    let template = ShowVideoTemplate {
        video,
        player_template: PlayerTemplate::from_saved(video_id),
    };

    Ok(BaseTemplate::wrap(htmx_request, template))
}

/// Returns template which displays all videos
///
/// `GET /`
///
/// # Returns
/// `VideoListTemplate` - template with list of all videos
pub async fn list_videos(htmx_request: HtmxRequest) -> impl Responder {
    BaseTemplate::wrap(htmx_request, VideoListTemplate {})
}

/// Returns template with create new video form
///
/// `GET /video/new`
///
/// # Returns
/// `VideoUploadTemplate`
pub async fn upload_video_template(
    htmx_request: HtmxRequest,
    config: Data<Configuration>,
) -> impl Responder {
    let video_input = VideoUploadInputTemplate::new(config.clone().into_inner());
    let thumbnail_input = ThumbnailUploadInputTemplate::new(config.into_inner());
    BaseTemplate::wrap(
        htmx_request,
        VideoUploadTemplate {
            video_input,
            thumbnail_input,
        },
    )
}

/// Returns template with edit video form
///
/// `GET /video/{id}/edit`
///
/// # Returns
/// `EditVideoTemplate`
pub async fn edit_video_template(
    params: Path<GetVideoByIdReq>,
    htmx_request: HtmxRequest,
    video_facade: Data<VideoFacade>,
) -> Result<impl Responder> {
    let video = video_facade.get_video_model(params.id, 1).await?;
    let video_input = VideoPreviewTemplate {
        temp_file_id: None,
        player_template: PlayerTemplate::from_saved(video.id),
    };
    let thumbnail_input = ThumbnailPreviewTemplate {
        temp_file_id: None,
        file_path: format!("/thumbnail/{}", video.id),
    };
    let template = BaseTemplate::wrap(
        htmx_request,
        EditVideoTemplate {
            video: video.into(),
            video_input,
            thumbnail_input,
        },
    );

    Ok(template.to_response())
}

fn add_redirect_header(path: &str, response: &mut HttpResponse) -> Result<()> {
    response.head_mut().headers.append(
        HeaderName::from_str("HX-Redirect").unwrap(),
        HeaderValue::from_str(path)?,
    );

    Ok(())
}
