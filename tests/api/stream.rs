use crate::api::video::{create_upload_req, upload_video};
use crate::common::users::{CHARLES_ARTIST, JOHN_ARTIST, JOHN_NOT_ARTIST, JOHN_PAYING};
use crate::common::utils::{create_empty_cookie, extract_id, login_as};
use actix_http::body::{BoxBody, EitherBody, MessageBody};
use actix_http::header::TryIntoHeaderPair;
use actix_http::Request;
use actix_service::Service;
use actix_web::cookie::Cookie;
use actix_web::dev::ServiceResponse;
use actix_web::web::Header;
use actix_web::{test, Error};
use erotic_hub::business::models::stream::LiveStreamStart;
use erotic_hub::business::models::video::VideoVisibility;
use erotic_hub::common::tests::setup::AsyncContext;
use http::{Method, StatusCode};
use lazy_static::lazy_static;
use regex::Regex;
use test_context::test_context;

lazy_static! {
    static ref GET_ID_REGEX: Regex = Regex::new(r#"\/stream\/(\d+)\/watch"#).unwrap();
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test01_start_stream_owner(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let response = start_stream(cookie.clone(), None, None, &app).await;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Start of the new stream was not successful"
    );
    let stream_id = extract_stream_id(response).await;

    assert!(stream_id > 0, "ID of new stream should be bigger than zero");
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test02_start_stream_different_artist(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;
    let artist2 = login_as(CHARLES_ARTIST, &app).await;

    let response = start_stream(cookie.clone(), Some(artist2), None, &app).await;
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Non-author artist was able to start the stream"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test03_stop_stream_same_artist(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let response = start_stream(cookie.clone(), None, None, &app).await;
    let stream_id = extract_stream_id(response).await;
    let stop_stream_resp = stop_stream(stream_id, cookie, &app).await;

    assert_eq!(
        stop_stream_resp.status(),
        StatusCode::NO_CONTENT,
        "Owner was not able to stop the stream"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test04_stop_stream_different_artist(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;
    let artist2_cookie = login_as(CHARLES_ARTIST, &app).await;

    let response = start_stream(cookie.clone(), None, None, &app).await;
    let stream_id = extract_stream_id(response).await;
    let stop_stream_resp = stop_stream(stream_id, artist2_cookie, &app).await;

    assert_eq!(
        stop_stream_resp.status(),
        StatusCode::NOT_FOUND,
        "Different artist was able to stop the stream"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test05_watch_stream_visible_all(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let author_cookie = login_as(JOHN_ARTIST, &app).await;
    let another_artist = login_as(CHARLES_ARTIST, &app).await;
    let registered = login_as(JOHN_NOT_ARTIST, &app).await;
    let cookie_paying = login_as(JOHN_PAYING, &app).await;

    let response = start_stream(
        author_cookie.clone(),
        None,
        Some(VideoVisibility::All),
        &app,
    )
    .await;
    let stream_id = extract_stream_id(response).await;

    check_get_requests(
        stream_id,
        create_empty_cookie(),
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
    check_get_requests(
        stream_id,
        author_cookie,
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
    check_get_requests(
        stream_id,
        another_artist,
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
    check_get_requests(stream_id, registered, StatusCode::OK, StatusCode::OK, &app).await;
    check_get_requests(
        stream_id,
        cookie_paying,
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test06_watch_stream_visible_paying(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let author_cookie = login_as(JOHN_ARTIST, &app).await;
    let another_artist = login_as(CHARLES_ARTIST, &app).await;
    let registered = login_as(JOHN_NOT_ARTIST, &app).await;
    let cookie_paying = login_as(JOHN_PAYING, &app).await;

    let response = start_stream(
        author_cookie.clone(),
        None,
        Some(VideoVisibility::Paying),
        &app,
    )
    .await;
    let stream_id = extract_stream_id(response).await;

    check_get_requests(
        stream_id,
        create_empty_cookie(),
        StatusCode::NOT_FOUND,
        StatusCode::FORBIDDEN,
        &app,
    )
    .await;
    check_get_requests(
        stream_id,
        author_cookie,
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
    check_get_requests(
        stream_id,
        another_artist,
        StatusCode::NOT_FOUND,
        StatusCode::FORBIDDEN,
        &app,
    )
    .await;
    check_get_requests(
        stream_id,
        registered,
        StatusCode::NOT_FOUND,
        StatusCode::FORBIDDEN,
        &app,
    )
    .await;
    check_get_requests(
        stream_id,
        cookie_paying,
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test07_watch_stream_visible_registered(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let author_cookie = login_as(JOHN_ARTIST, &app).await;
    let another_artist = login_as(CHARLES_ARTIST, &app).await;
    let registered = login_as(JOHN_NOT_ARTIST, &app).await;
    let cookie_paying = login_as(JOHN_PAYING, &app).await;

    let response = start_stream(
        author_cookie.clone(),
        None,
        Some(VideoVisibility::Registered),
        &app,
    )
    .await;
    let stream_id = extract_stream_id(response).await;

    check_get_requests(
        stream_id,
        create_empty_cookie(),
        StatusCode::NOT_FOUND,
        StatusCode::FORBIDDEN,
        &app,
    )
    .await;
    check_get_requests(
        stream_id,
        author_cookie,
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
    check_get_requests(
        stream_id,
        another_artist,
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
    check_get_requests(stream_id, registered, StatusCode::OK, StatusCode::OK, &app).await;
    check_get_requests(
        stream_id,
        cookie_paying,
        StatusCode::OK,
        StatusCode::OK,
        &app,
    )
    .await;
}

async fn check_get_requests(
    stream_id: i32,
    cookie: Cookie<'_>,
    watch_status: StatusCode,
    authenticate_status: StatusCode,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) {
    let watch_stream_resp = watch_stream(stream_id, cookie.clone(), &app).await;
    assert_eq!(watch_stream_resp.status(), watch_status);

    let authenticate_stream_resp = authenticate_stream(stream_id, cookie, &app).await;
    assert_eq!(authenticate_stream_resp.status(), authenticate_status);
}

async fn start_stream(
    cookie: Cookie<'_>,
    upload_video_cookie: Option<Cookie<'_>>,
    visibility: Option<VideoVisibility>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let mut req = create_upload_req(
        String::from("Vid"),
        visibility.unwrap_or(VideoVisibility::All),
        Some(String::from("Description")),
    );

    let video_id = extract_id(
        upload_video(
            &mut req,
            None,
            None,
            upload_video_cookie.unwrap_or(cookie.clone()),
            &app,
        )
        .await,
    )
    .await;

    let start_req = LiveStreamStart { video_id };
    let request = test::TestRequest::default()
        .uri("/stream/start")
        .method(Method::POST)
        .set_form(start_req)
        .cookie(cookie);

    request.send_request(&app).await
}

async fn stop_stream(
    stream_id: i32,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let request = test::TestRequest::default()
        .uri(&format!("/stream/{stream_id}/stop"))
        .method(Method::DELETE)
        .cookie(cookie);

    request.send_request(&app).await
}

async fn watch_stream(
    stream_id: i32,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let request = test::TestRequest::default()
        .uri(&format!("/stream/{stream_id}/watch"))
        .method(Method::GET)
        .cookie(cookie);

    request.send_request(&app).await
}

async fn authenticate_stream(
    stream_id: i32,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let request = test::TestRequest::default()
        .uri("/stream/auth")
        .method(Method::GET)
        .insert_header(("X-Original-URI", format!("/hls/stream-{stream_id}.m3u8")))
        .cookie(cookie);

    request.send_request(&app).await
}

async fn extract_stream_id(response: ServiceResponse<EitherBody<BoxBody>>) -> i32 {
    let stream_path = response
        .headers()
        .get("HX-Redirect")
        .unwrap()
        .to_str()
        .unwrap();
    let captures = GET_ID_REGEX.captures(stream_path).unwrap();

    captures.get(1).unwrap().as_str().parse().unwrap()
}
