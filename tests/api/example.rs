use crate::common::files;
use crate::common::users::JOHN_ARTIST;
use crate::common::utils::login_as;
use actix_web::http::{Method, StatusCode};
use actix_web::test;
use erotic_hub::common::tests::setup::AsyncContext;
use test_context::test_context;

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test01(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let req = test::TestRequest::default().uri("/user/login");

    let response = req.send_request(&app).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test02(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let (header, body) = files::VIDEO1.get_multipart_builder("file", "video/mp4");
    let req = test::TestRequest::default()
        .uri("/temp/video")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .cookie(cookie);

    let response = req.send_request(&app).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}
