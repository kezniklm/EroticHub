use crate::api::controllers::utils::route_util::{add_redirect_header, build_stream_watch_path};
use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::extractors::permissions_extractor::{AsInteger, AsIntegerOptional};
use crate::api::templates::stream::watch::template::WatchStreamTemplate;
use crate::api::templates::template::BaseTemplate;
use crate::business::facades::stream::{StreamFacade, StreamFacadeTrait};
use crate::business::models::error::{AppError, AppErrorKind};
use crate::business::models::stream::LiveStreamStart;
use crate::business::models::user::UserRole::{self, Artist};
use actix_identity::Identity;
use actix_session::Session;
use actix_web::web::{Data, Form, Path};
use actix_web::{HttpRequest, HttpResponse, Responder, Result};
use actix_web_grants::protect;
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
#[protect(any("Artist"), ty = "UserRole")]
pub async fn start_stream(
    Form(request): Form<LiveStreamStart>,
    stream_facade: Data<StreamFacade>,
    identity: Identity,
) -> Result<impl Responder> {
    let stream_id = stream_facade
        .start_stream(request, identity.id_i32()?)
        .await?;

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
    session: Session,
    htmx_request: HtmxRequest,
    identity: Option<Identity>,
) -> Result<impl Responder> {
    let (video, stream) = stream_facade
        .get_stream(identity.id_i32(), stream_id.into_inner())
        .await?;
    let template = BaseTemplate::wrap(htmx_request, session, WatchStreamTemplate { stream, video });

    Ok(template.to_response())
}

/// Stops the running stream
///
/// `DELETE /stream/{stream_id}/stop`
///
/// # Returns
/// Redirects user to the main page
#[protect(any("Artist"), ty = "UserRole")]
pub async fn stop_stream(
    stream_id: Path<i32>,
    stream_facade: Data<StreamFacade>,
    identity: Identity,
) -> Result<impl Responder> {
    stream_facade
        .stop_stream(identity.id_i32()?, stream_id.into_inner())
        .await?;

    let mut response = HttpResponse::NoContent().finish();

    add_redirect_header("/", &mut response)?;
    Ok(response)
}

/// Checks if user can access the stream
///
/// `GET /stream/auth`
///
/// # Returns
/// HTTP 200 if yes, HTTP 403 if access was denied
pub async fn authenticate_stream_request(
    request: HttpRequest,
    stream_facade: Data<StreamFacade>,
    identity: Option<Identity>,
) -> Result<impl Responder> {
    let headers = request.headers();
    let stream_url = headers.get("X-Original-URI").ok_or(AppError::new(
        "Access to the stream denied!",
        AppErrorKind::AccessDenied,
    ))?;

    let stream_url = stream_url
        .to_str()
        .map_err(|_| AppError::new("Access to the stream denied!", AppErrorKind::AccessDenied))?;

    stream_facade
        .authenticate_stream(identity.id_i32(), stream_url)
        .await
        // Map all errors to 403 Forbidden, since it's needed by Nginx plugin
        .map_err(|err| AppError::new(&err.message, AppErrorKind::AccessDenied))?;

    Ok(HttpResponse::Ok().finish())
}
