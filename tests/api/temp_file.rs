use crate::common::files;
use crate::common::files::TestFile;
use crate::common::users::{CHARLES_ARTIST, JOHN_ARTIST, JOHN_NOT_ARTIST};
use crate::common::utils::{extract_id, login_as};
use actix_http::body::{BoxBody, EitherBody};
use actix_http::Request;
use actix_service::Service;
use actix_web::cookie::Cookie;
use actix_web::dev::ServiceResponse;
use actix_web::{test, Error};
use erotic_hub::common::tests::setup::AsyncContext;
use http::{Method, StatusCode};
use test_context::test_context;

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test01_upload_video_artist(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let temp_file_req = upload_temp_video(cookie, None, &app).await;
    assert_eq!(
        temp_file_req.status(),
        StatusCode::CREATED,
        "Temp file was not uploaded"
    );
    let temp_file_id = extract_id(temp_file_req).await;
    assert!(
        temp_file_id > 0,
        "Temporary file ID should be bigger than zero"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test02_upload_video_non_artist(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_NOT_ARTIST, &app).await;

    let (header, body) = files::VIDEO1.get_multipart_builder("file", "video/mp4");
    let req = test::TestRequest::default()
        .uri("/temp/video")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie);

    let response = req.send_request(&app).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test03_upload_thumbnail_artist(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let (header, body) = files::PLACEHOLDER_PNG.get_multipart_builder("file", "image/png");
    let req = test::TestRequest::default()
        .uri("/temp/thumbnail")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie);

    let response = req.send_request(&app).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test04_upload_video_non_artist(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_NOT_ARTIST, &app).await;

    let (header, body) = files::PLACEHOLDER_PNG.get_multipart_builder("file", "image/png");
    let req = test::TestRequest::default()
        .uri("/temp/thumbnail")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie);

    let response = req.send_request(&app).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test05_get_temp_file_different_user(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie_john = login_as(JOHN_ARTIST, &app).await;
    let cookie_charles = login_as(CHARLES_ARTIST, &app).await;

    let temp_file_id = extract_id(upload_temp_video(cookie_john, None, &app).await).await;

    let req = test::TestRequest::default()
        .uri(&format!("/temp/{temp_file_id}"))
        .method(Method::GET)
        .cookie(cookie_charles);

    let response = req.send_request(&app).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test06_get_temp_file_same_user(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie_john = login_as(JOHN_ARTIST, &app).await;

    let temp_file_id = extract_id(upload_temp_video(cookie_john.clone(), None, &app).await).await;

    let response = get_temp_file_res(temp_file_id, cookie_john, &app).await;
    assert_eq!(response.status(), StatusCode::OK);

    let fs_video = tokio::fs::read(files::VIDEO1.get_path_to_file())
        .await
        .expect("Failed to load the video from filesystem");
    let response_video = test::read_body(response).await.to_vec();
    assert_eq!(
        response_video, fs_video,
        "Returned different video than uploaded"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test07_delete_temp_file_different_user(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie_john = login_as(JOHN_ARTIST, &app).await;
    let cookie_charles = login_as(CHARLES_ARTIST, &app).await;

    let temp_file_id = extract_id(upload_temp_video(cookie_john.clone(), None, &app).await).await;

    let req = test::TestRequest::default()
        .uri(&format!("/temp/{temp_file_id}?input_type=Video"))
        .method(Method::DELETE)
        .cookie(cookie_charles);

    let response = req.send_request(&app).await;
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "File can be deleted with different user"
    );

    let get_file_resp = get_temp_file_res(temp_file_id, cookie_john, &app).await;
    assert_eq!(
        get_file_resp.status(),
        StatusCode::OK,
        "Temp file was deleted by different user!"
    )
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test08_delete_temp_file_same_user(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie_john = login_as(JOHN_ARTIST, &app).await;

    let temp_file_id = extract_id(upload_temp_video(cookie_john.clone(), None, &app).await).await;

    let req = test::TestRequest::default()
        .uri(&format!("/temp/{temp_file_id}?input_type=Video"))
        .method(Method::DELETE)
        .cookie(cookie_john.clone());

    let response = req.send_request(&app).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Temporary file cannot be deleted"
    );

    let get_file_resp = get_temp_file_res(temp_file_id, cookie_john, &app).await;
    assert_eq!(
        get_file_resp.status(),
        StatusCode::NOT_FOUND,
        "Temp file was deleted by different user!"
    )
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test09_upload_video_wrong_mimetype(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie_john = login_as(JOHN_ARTIST, &app).await;

    let (header, body) = files::PLACEHOLDER_PNG.get_multipart_builder("file", "image/png");
    let req = test::TestRequest::default()
        .uri("/temp/video?get_template=false")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie_john);

    let response = req.send_request(&app).await;

    assert_eq!(
        response.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "File was accepted with unsupported media type"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test10_upload_video_different_mimetype(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie_john = login_as(JOHN_ARTIST, &app).await;

    let (header, body) = files::PLACEHOLDER_PNG.get_multipart_builder("file", "video/mp4");
    let req = test::TestRequest::default()
        .uri("/temp/video?get_template=false")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie_john);

    let response = req.send_request(&app).await;

    assert_eq!(
        response.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "File was accepted with different media type"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test11_upload_thumbnail_wrong_mimetype(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie_john = login_as(JOHN_ARTIST, &app).await;

    let (header, body) = files::VIDEO1.get_multipart_builder("file", "video/mp4");
    let req = test::TestRequest::default()
        .uri("/temp/thumbnail?get_template=false")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie_john);

    let response = req.send_request(&app).await;

    assert_eq!(
        response.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "File was accepted with unsupported media type"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test12_upload_thumbnail_different_mimetype(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie_john = login_as(JOHN_ARTIST, &app).await;

    let (header, body) = files::VIDEO1.get_multipart_builder("file", "image/png");
    let req = test::TestRequest::default()
        .uri("/temp/thumbnail?get_template=false")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie_john);

    let response = req.send_request(&app).await;

    assert_eq!(
        response.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "File was accepted with different media type"
    );
}

pub async fn upload_temp_video(
    cookie: Cookie<'_>,
    file: Option<TestFile>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let (header, body) = match file {
        None => files::VIDEO1.get_multipart_builder("file", "video/mp4"),
        Some(file) => file.get_multipart_builder("file", "video/mp4"),
    };
    let req = test::TestRequest::default()
        .uri("/temp/video?get_template=false")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie);

    req.send_request(&app).await
}

pub async fn upload_temp_thumbnail(
    cookie: Cookie<'_>,
    file: Option<TestFile>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let (header, body) = match file {
        None => files::PLACEHOLDER_PNG.get_multipart_builder("file", "image/png"),
        Some(file) => file.get_multipart_builder("file", "image/png"),
    };
    let req = test::TestRequest::default()
        .uri("/temp/thumbnail?get_template=false")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie);

    req.send_request(&app).await
}

async fn get_temp_file_res(
    temp_file_id: i32,
    cookie: Cookie<'_>,
    app: impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> ServiceResponse<EitherBody<BoxBody>> {
    let req = test::TestRequest::default()
        .uri(&format!("/temp/{temp_file_id}"))
        .method(Method::GET)
        .cookie(cookie);

    req.send_request(&app).await
}
