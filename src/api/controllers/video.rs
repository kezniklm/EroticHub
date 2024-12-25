use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::template::BaseTemplate;
use crate::api::templates::video::list::template::VideoListTemplate;
use crate::api::templates::video::upload::template::{
    ThumbnailUploadInputTemplate, VideoUploadInputTemplate, VideoUploadTemplate,
};
use crate::business::facades::video::{VideoFacade, VideoFacadeTrait};
use crate::business::models::video::{PlayableVideoReq, VideoUploadData};
use crate::configuration::models::Configuration;
use actix_files::NamedFile;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Query};
use actix_web::{web, Error, HttpResponse, Responder, Result};
use log::error;

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

pub async fn list_videos(htmx_request: HtmxRequest) -> impl Responder {
    BaseTemplate::wrap(htmx_request, VideoListTemplate {})
}

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
