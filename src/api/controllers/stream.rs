use crate::api::controllers::utils::route_util::{add_redirect_header, build_stream_watch_path};
use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::stream::watch::template::WatchStreamTemplate;
use crate::api::templates::template::BaseTemplate;
use crate::business::facades::stream::{StreamFacade, StreamFacadeTrait};
use crate::business::models::stream::LiveStreamStart;
use actix_web::web::{Data, Form, Path};
use actix_web::{HttpResponse, Responder, Result};
use askama_actix::TemplateToResponse;

/// Starts the live stream
///
/// `POST /stream/start`
///
/// # Form params
/// `LiveStreamStart` - if of the video to be streamed
///
/// # Returns
/// Redirects user to the started stream
pub async fn start_stream(
    Form(request): Form<LiveStreamStart>,
    stream_facade: Data<StreamFacade>,
) -> Result<impl Responder> {
    let stream_id = stream_facade.start_stream(request, 2).await?;

    let mut response = HttpResponse::Created().finish();
    add_redirect_header(&build_stream_watch_path(stream_id), &mut response)?;
    Ok(response)
}

/// Returns watch stream template
///
/// `GET /stream/{stream_id}/watch`
///
/// # Returns
/// HTML template with stream player
pub async fn watch_stream(
    stream_id: Path<i32>,
    stream_facade: Data<StreamFacade>,
    htmx_request: HtmxRequest,
) -> Result<impl Responder> {
    let (video, stream) = stream_facade.get_stream(1, stream_id.into_inner()).await?;
    let template = BaseTemplate::wrap(htmx_request, WatchStreamTemplate { stream, video });

    Ok(template.to_response())
}

/// Stops the running stream
/// TODO: Permissions check!
///
/// `DELETE /stream/{stream_id}/stop`
///
/// # Returns
/// Redirects user to the main page
pub async fn stop_stream(
    stream_id: Path<i32>,
    stream_facade: Data<StreamFacade>,
) -> Result<impl Responder> {
    stream_facade.stop_stream(1, stream_id.into_inner()).await?;

    let mut response = HttpResponse::NoContent().finish();

    add_redirect_header("/", &mut response)?;
    Ok(response)
}
