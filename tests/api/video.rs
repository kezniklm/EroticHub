use crate::api::temp_file::{upload_temp_thumbnail, upload_temp_video};
use crate::common::files;
use crate::common::files::{TestFile, VIDEO2};
use crate::common::users::{CHARLES_ARTIST, JOHN_ARTIST, JOHN_NOT_ARTIST, JOHN_PAYING};
use crate::common::utils::{create_empty_cookie, extract_id, login_as};
use actix_http::body::{BoxBody, EitherBody};
use actix_http::Request;
use actix_service::Service;
use actix_web::cookie::Cookie;
use actix_web::dev::ServiceResponse;
use actix_web::{test, Error};
use erotic_hub::business::models::video::{VideoEditReq, VideoUploadReq, VideoVisibility};
use erotic_hub::common::tests::setup::AsyncContext;
use http::{Method, StatusCode};
use test_context::test_context;

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test01_upload_artist(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(
        String::from("Vid"),
        VideoVisibility::All,
        Some(String::from("Description")),
    );

    let response = upload_video(&mut req, None, None, cookie.clone(), &app).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let video_id = extract_id(response).await;
    assert!(video_id > 0, "ID of the new video must be bigger than zero");

    let watch_res = watch_video(video_id, cookie, &app).await;
    assert_eq!(
        watch_res.status(),
        StatusCode::OK,
        "Video was not saved correctly!"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test02_upload_short_name(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(
        String::from("Vi"),
        VideoVisibility::All,
        Some(String::from("Description")),
    );

    let response = upload_video(&mut req, None, None, cookie, &app).await;
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Video can be uploaded with short name"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test03_upload_long_name(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(
        "x".repeat(128),
        VideoVisibility::All,
        Some(String::from("Description")),
    );

    let response = upload_video(&mut req, None, None, cookie.clone(), &app).await;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Video cannot be uploaded with standard name"
    );

    let mut req = create_upload_req(
        "x".repeat(129),
        VideoVisibility::All,
        Some(String::from("Description")),
    );
    let response = upload_video(&mut req, None, None, cookie, &app).await;
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Video can be uploaded with long name"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test04_upload_wrong_temp_video(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(
        String::from("Video"),
        VideoVisibility::All,
        Some(String::from("Description")),
    );

    let response = upload_video(&mut req, Some(74475847), None, cookie.clone(), &app).await;
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Video can be uploaded with wrong temp file id"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test05_upload_wrong_temp_thumbnail(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(
        String::from("Video"),
        VideoVisibility::All,
        Some(String::from("Description")),
    );

    let response = upload_video(&mut req, None, Some(74475847), cookie.clone(), &app).await;
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Video can be uploaded with wrong temp file id"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test06_upload_long_description(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(
        String::from("Video"),
        VideoVisibility::All,
        Some("D".repeat(5000)),
    );

    let response = upload_video(&mut req, None, None, cookie.clone(), &app).await;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Video cannot be uploaded with allowed length of description"
    );

    let mut req = create_upload_req(
        String::from("x".repeat(129)),
        VideoVisibility::All,
        Some("D".repeat(5001)),
    );
    let response = upload_video(&mut req, None, None, cookie, &app).await;
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Video can be uploaded with long description"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test07_upload_long_description(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(String::from("x".repeat(129)), VideoVisibility::All, None);
    let response = upload_video(&mut req, None, None, cookie, &app).await;
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Video cannot be uploaded with no description"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test08_visibility_all(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let artist_cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(String::from("Video"), VideoVisibility::All, None);

    let video_id =
        extract_id(upload_video(&mut req, None, None, artist_cookie.clone(), &app).await).await;
    assert_get_requests(video_id, StatusCode::OK, create_empty_cookie(), &app).await;
    assert_get_requests(video_id, StatusCode::OK, artist_cookie, &app).await;

    let paying_cookie = login_as(JOHN_PAYING, &app).await;
    assert_get_requests(video_id, StatusCode::OK, paying_cookie, &app).await;

    let registered_cookie = login_as(JOHN_NOT_ARTIST, &app).await;
    assert_get_requests(video_id, StatusCode::OK, registered_cookie, &app).await;
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test09_visibility_paying(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    // Both artists are not paying members!
    let artist_cookie = login_as(JOHN_ARTIST, &app).await;
    let not_owner_artist = login_as(CHARLES_ARTIST, &app).await;

    let mut req = create_upload_req(String::from("Video"), VideoVisibility::Paying, None);

    let video_id =
        extract_id(upload_video(&mut req, None, None, artist_cookie.clone(), &app).await).await;
    assert_get_requests(video_id, StatusCode::NOT_FOUND, create_empty_cookie(), &app).await;
    assert_get_requests(video_id, StatusCode::OK, artist_cookie, &app).await;
    assert_get_requests(video_id, StatusCode::NOT_FOUND, not_owner_artist, &app).await;

    let paying_cookie = login_as(JOHN_PAYING, &app).await;
    assert_get_requests(video_id, StatusCode::OK, paying_cookie, &app).await;

    let registered_cookie = login_as(JOHN_NOT_ARTIST, &app).await;
    assert_get_requests(video_id, StatusCode::NOT_FOUND, registered_cookie, &app).await;
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test10_visibility_registered(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let artist_cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(String::from("Video"), VideoVisibility::Registered, None);

    let video_id =
        extract_id(upload_video(&mut req, None, None, artist_cookie.clone(), &app).await).await;
    assert_get_requests(video_id, StatusCode::NOT_FOUND, create_empty_cookie(), &app).await;
    assert_get_requests(video_id, StatusCode::OK, artist_cookie, &app).await;

    let paying_cookie = login_as(JOHN_PAYING, &app).await;
    assert_get_requests(video_id, StatusCode::OK, paying_cookie, &app).await;

    let registered_cookie = login_as(JOHN_NOT_ARTIST, &app).await;
    assert_get_requests(video_id, StatusCode::OK, registered_cookie, &app).await;
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test11_patch_different_user(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let artist_cookie = login_as(JOHN_ARTIST, &app).await;
    let artist2_cookie = login_as(CHARLES_ARTIST, &app).await;

    let mut req = create_upload_req(String::from("Video"), VideoVisibility::Registered, None);
    let video_id =
        extract_id(upload_video(&mut req, None, None, artist_cookie.clone(), &app).await).await;

    let mut patch_video_req = VideoEditReq {
        name: Some(String::from("New name")),
        video_visibility: VideoVisibility::All,
        temp_thumbnail_id: None,
        temp_video_id: None,
        description: None,
    };

    let response = patch_video(
        video_id,
        &mut patch_video_req,
        Some(VIDEO2),
        None,
        artist2_cookie,
        &app,
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Video can be updated only by its author"
    );

    let video_file = get_video_file(video_id, artist_cookie, &app).await;
    let fs_video = tokio::fs::read(files::VIDEO1.get_path_to_file())
        .await
        .expect("Failed to load the video from filesystem");
    let response_video = test::read_body(video_file).await.to_vec();

    assert_eq!(
        response_video, fs_video,
        "Video should not be updated by different user"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test12_patch_same_user(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let artist_cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(String::from("Video"), VideoVisibility::Registered, None);
    let video_id =
        extract_id(upload_video(&mut req, None, None, artist_cookie.clone(), &app).await).await;

    let mut patch_video_req = VideoEditReq {
        name: Some(String::from("New name")),
        video_visibility: VideoVisibility::All,
        temp_thumbnail_id: None,
        temp_video_id: None,
        description: None,
    };

    let response = patch_video(
        video_id,
        &mut patch_video_req,
        Some(VIDEO2),
        None,
        artist_cookie.clone(),
        &app,
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Video can be updated only by its author"
    );

    let video_file = get_video_file(video_id, artist_cookie, &app).await;
    let fs_video = tokio::fs::read(VIDEO2.get_path_to_file())
        .await
        .expect("Failed to load the video from filesystem");
    let response_video = test::read_body(video_file).await.to_vec();

    assert_eq!(response_video, fs_video, "Video should be updated");
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test13_delete_same_user(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let artist_cookie = login_as(JOHN_ARTIST, &app).await;

    let mut req = create_upload_req(String::from("Video"), VideoVisibility::Registered, None);
    let video_id =
        extract_id(upload_video(&mut req, None, None, artist_cookie.clone(), &app).await).await;

    let response = delete_video(video_id, artist_cookie.clone(), &app).await;

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "Video cannot be delete by its author"
    );

    let video_file_res = get_video_file(video_id, artist_cookie, &app).await;

    assert_eq!(
        video_file_res.status(),
        StatusCode::NOT_FOUND,
        "Video was not deleted"
    )
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test13_delete_different_user(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let artist_cookie = login_as(JOHN_ARTIST, &app).await;
    let artist2_cookie = login_as(CHARLES_ARTIST, &app).await;

    let mut req = create_upload_req(String::from("Video"), VideoVisibility::Registered, None);
    let video_id =
        extract_id(upload_video(&mut req, None, None, artist_cookie.clone(), &app).await).await;

    let response = delete_video(video_id, artist2_cookie, &app).await;

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Video can be deleted only by its author"
    );

    let video_file_res = get_video_file(video_id, artist_cookie, &app).await;

    assert_eq!(
        video_file_res.status(),
        StatusCode::OK,
        "Video was deleted by different user"
    )
}

async fn assert_get_requests(
    video_id: i32,
    expected_status: StatusCode,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) {
    let watch_video_resp = watch_video(video_id, cookie.clone(), &app).await;

    assert_eq!(watch_video_resp.status(), expected_status);

    let video_file_resp = get_video_file(video_id, cookie.clone(), &app).await;

    assert_eq!(video_file_resp.status(), expected_status);

    let thumbnail_file_resp = get_thumbnail_file(video_id, cookie, &app).await;

    assert_eq!(thumbnail_file_resp.status(), expected_status);
}

pub fn create_upload_req(
    name: String,
    video_visibility: VideoVisibility,
    description: Option<String>,
) -> VideoUploadReq {
    VideoUploadReq {
        name,
        video_visibility,
        temp_thumbnail_id: -1,
        temp_video_id: -1,
        description,
    }
}

pub async fn upload_video(
    upload_video_form: &mut VideoUploadReq,
    video_temp_id: Option<i32>,
    thumbnail_temp_id: Option<i32>,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    upload_video_form.temp_video_id = video_temp_id
        .unwrap_or(extract_id(upload_temp_video(cookie.clone(), None, &app).await).await);
    upload_video_form.temp_thumbnail_id = thumbnail_temp_id
        .unwrap_or(extract_id(upload_temp_thumbnail(cookie.clone(), None, &app).await).await);

    let request = test::TestRequest::default()
        .uri("/video?get_template=false")
        .method(Method::POST)
        .set_form(upload_video_form)
        .cookie(cookie);

    request.send_request(&app).await
}

async fn patch_video(
    video_id: i32,
    video_edit_req: &mut VideoEditReq,
    new_video_file: Option<TestFile>,
    new_thumbnail_file: Option<TestFile>,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    video_edit_req.temp_video_id = match new_video_file {
        None => None,
        Some(file) => {
            Some(extract_id(upload_temp_video(cookie.clone(), Some(file), &app).await).await)
        }
    };

    video_edit_req.temp_thumbnail_id = match new_thumbnail_file {
        None => None,
        Some(file) => {
            Some(extract_id(upload_temp_thumbnail(cookie.clone(), Some(file), &app).await).await)
        }
    };

    let request = test::TestRequest::default()
        .uri(&format!("/video/{video_id}"))
        .method(Method::PATCH)
        .set_form(video_edit_req)
        .cookie(cookie);

    request.send_request(&app).await
}

async fn delete_video(
    video_id: i32,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let request = test::TestRequest::default()
        .uri(&format!("/video/{video_id}"))
        .method(Method::DELETE)
        .cookie(cookie);

    request.send_request(&app).await
}

async fn watch_video(
    video_id: i32,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let request = test::TestRequest::default()
        .uri(&format!("/video/{video_id}/watch"))
        .method(Method::GET)
        .cookie(cookie);

    request.send_request(&app).await
}

async fn get_video_file(
    video_id: i32,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let request = test::TestRequest::default()
        .uri(&format!("/video/{video_id}"))
        .method(Method::GET)
        .cookie(cookie);

    request.send_request(&app).await
}

async fn get_thumbnail_file(
    video_id: i32,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let request = test::TestRequest::default()
        .uri(&format!("/thumbnail/{video_id}"))
        .method(Method::GET)
        .cookie(cookie);

    request.send_request(&app).await
}
