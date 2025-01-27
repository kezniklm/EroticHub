use crate::api::controllers::utils::route_util::{
    add_redirect_header, build_get_video_path, build_watch_path,
};
use crate::api::controllers::utils::video_utils::parse_option_string;
use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::extractors::permissions_extractor::{AsInteger, AsIntegerOptional};
use crate::api::extractors::template_extractor::TemplateReq;
use crate::api::templates::template::BaseTemplate;
use crate::api::templates::video::edit::template::EditVideoTemplate;
use crate::api::templates::video::list::template::{
    IndexTemplate, VideoGridTemplate, VideosTemplate,
};
use crate::api::templates::video::show::template::{PlayerTemplate, ShowVideoTemplate};
use crate::api::templates::video::upload::template::{
    ThumbnailPreviewTemplate, ThumbnailUploadInputTemplate, VideoPreviewTemplate,
    VideoUploadInputTemplate, VideoUploadTemplate,
};
use crate::business::facades::artist::{ArtistFacade, ArtistFacadeTrait};
use crate::business::facades::video::{VideoFacade, VideoFacadeTrait};
use crate::business::facades::video_category::{VideoCategoryFacade, VideoCategoryFacadeTrait};
use crate::business::models::user::UserRole::{self, Artist};
use crate::business::models::video::{
    FetchVideoByFilters, GetVideoByIdReq, VideoEditReq, VideoList, VideoUploadReq,
};
use crate::configuration::models::Configuration;
use actix_files::NamedFile;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::web::{Data, Path, Query};
use actix_web::{HttpResponse, Responder, Result};
use actix_web_grants::protect;
use anyhow::anyhow;
use askama_actix::TemplateToResponse;
use serde_qs::actix::QsForm;

/// Creates new video
///
/// `POST /video`
///
/// # Form params
/// `VideoUploadReq` - data of uploaded video
///
/// # Returns
/// Redirects user to the newly created video
#[protect(any("Artist"), ty = "UserRole")]
pub async fn create_video(
    form: QsForm<VideoUploadReq>,
    video_facade: Data<VideoFacade>,
    template_req: TemplateReq,
    identity: Identity,
) -> Result<impl Responder> {
    let video = video_facade
        .save_video(identity.id_i32()?, form.into_inner())
        .await?;
    let video_id = video.id;

    if !template_req.return_template {
        return Ok(HttpResponse::Created().body(video_id.to_string()));
    }

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
#[protect(any("Artist"), ty = "UserRole")]
pub async fn patch_video(
    path: Path<GetVideoByIdReq>,
    form: QsForm<VideoEditReq>,
    video_facade: Data<VideoFacade>,
    identity: Identity,
) -> Result<impl Responder> {
    let video = video_facade
        .patch_video(identity.id_i32()?, path.id, form.into_inner())
        .await?;

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
#[protect(any("Artist"), ty = "UserRole")]
pub async fn delete_video(
    path: Path<GetVideoByIdReq>,
    video_facade: Data<VideoFacade>,
    identity: Identity,
) -> Result<impl Responder> {
    video_facade
        .delete_video(identity.id_i32()?, path.id)
        .await?;

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
    identity: Option<Identity>,
) -> Result<NamedFile> {
    let file = video_facade
        .get_playable_video(request.id, identity.id_i32())
        .await?;
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
    identity: Option<Identity>,
) -> Result<NamedFile> {
    let file = video_facade
        .get_thumbnail_file(request.id, identity.id_i32())
        .await?;
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
    session: Session,
    identity: Option<Identity>,
) -> Result<impl Responder> {
    let video = video_facade
        .get_video_model(req.id, identity.id_i32())
        .await?;
    let video_id = video.id;
    let template = ShowVideoTemplate {
        video,
        player_template: PlayerTemplate::from_saved(video_id),
    };

    Ok(BaseTemplate::wrap(htmx_request, session, template))
}

pub async fn main_page(
    video_facade: Data<VideoFacade>,
    artist_facade: Data<ArtistFacade>,
    category_facade: Data<VideoCategoryFacade>,
    req: Query<FetchVideoByFilters>,
    session: Session,
    htmx_request: HtmxRequest,
) -> impl Responder {
    let serialized_videos = get_videos(video_facade, artist_facade, req.clone()).await;

    let serialized_videos = match serialized_videos {
        Ok(videos) => videos,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let categories = category_facade.list_categories().await;

    let categories = match categories {
        Ok(categories) => categories,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let template = IndexTemplate {
        videos_template: VideoGridTemplate {
            videos: serialized_videos,
        },
        categories,
    };

    BaseTemplate::wrap(htmx_request, session, template).to_response()
}

pub async fn list_videos(
    video_facade: Data<VideoFacade>,
    artist_facade: Data<ArtistFacade>,
    req: Query<FetchVideoByFilters>,
) -> impl Responder {
    let serialized_videos = get_videos(video_facade, artist_facade, req.clone()).await;

    let serialized_videos = match serialized_videos {
        Ok(videos) => videos,
        Err(e) => {
            if e.to_string() == "No videos" {
                return HttpResponse::NoContent().json("No videos");
            } else {
                return HttpResponse::InternalServerError().json(e.to_string());
            }
        }
    };

    let template = VideosTemplate {
        videos: serialized_videos,
    };

    template.to_response()
}

pub async fn get_videos(
    video_facade: Data<VideoFacade>,
    artist_facade: Data<ArtistFacade>,
    req: Query<FetchVideoByFilters>,
) -> anyhow::Result<Vec<VideoList>> {
    let offset = req.offset;
    let filter: Option<Vec<i32>> = parse_option_string(req.filter.clone())?;
    let ord = req.ord.as_deref();

    let videos = video_facade.fetch_videos(ord, filter, offset).await;

    let videos = match videos {
        Ok(videos) => {
            if videos.is_empty() {
                return Err(anyhow::anyhow!("No videos"));
            } else {
                videos
            }
        }
        Err(e) => return Err(anyhow!(e.to_string())),
    };

    let mut artists_ids = Vec::new();
    videos.iter().for_each(|v| {
        artists_ids.push(v.artist_id);
    });

    let artists = artist_facade.get_artists_names_by_id(artists_ids).await?;

    let mut serialized_videos = Vec::new();
    for video in &videos {
        for artist in &artists {
            if video.artist_id == artist.id {
                let (_video_path, thumbnail_path) = build_get_video_path(video.id);
                serialized_videos.push(VideoList {
                    id: video.id,
                    artist_id: video.artist_id,
                    artist_name: artist.name.clone(),
                    thumbnail_path,
                    name: video.name.clone(),
                })
            }
        }
    }

    Ok(serialized_videos)
}

/// Returns template with create new video form
///
/// `GET /video/new`
///
/// # Returns
/// `VideoUploadTemplate`
#[protect(any("Artist"), ty = "UserRole")]
pub async fn upload_video_template(
    htmx_request: HtmxRequest,
    session: Session,
    config: Data<Configuration>,
    category_facade: Data<VideoCategoryFacade>,
) -> Result<impl Responder> {
    let video_input = VideoUploadInputTemplate::new(config.clone().into_inner());
    let thumbnail_input = ThumbnailUploadInputTemplate::new(config.into_inner());
    let template = BaseTemplate::wrap(
        htmx_request,
        session,
        VideoUploadTemplate {
            video_input,
            thumbnail_input,
            categories: category_facade.list_categories().await?,
        },
    );

    Ok(template.to_response())
}

/// Returns template with edit video form
///
/// `GET /video/{id}/edit`
///
/// # Returns
/// `EditVideoTemplate`
#[protect(any("Artist"), ty = "UserRole")]
pub async fn edit_video_template(
    params: Path<GetVideoByIdReq>,
    htmx_request: HtmxRequest,
    session: Session,
    video_facade: Data<VideoFacade>,
    video_category_facade: Data<VideoCategoryFacade>,
    identity: Option<Identity>,
) -> Result<impl Responder> {
    let video = video_facade
        .get_video_model(params.id, identity.id_i32())
        .await?;
    let categories = video_category_facade
        .get_selected_categories(params.id)
        .await?;
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
        session,
        EditVideoTemplate {
            video: video.into(),
            video_input,
            thumbnail_input,
            categories,
        },
    );

    Ok(template.to_response())
}
