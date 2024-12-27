use crate::common::files;
use actix_web::http::{Method, StatusCode};
use actix_web::{test, App};
use erotic_hub::common::tests::setup::AsyncContext;
use test_context::test_context;

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test01(ctx: &mut AsyncContext) {
    println!("test01 start");
    let app = test::init_service(App::new().configure(ctx.configure_app())).await;
    let req = test::TestRequest::default().uri("/user");

    let response = req.send_request(&app).await;
    assert_eq!(response.status(), StatusCode::OK);
    println!("test01 end");
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test02(ctx: &mut AsyncContext) {
    println!("test02 start");
    let app = test::init_service(App::new().configure(ctx.configure_app())).await;

    let (header, body) = files::VIDEO1.get_multipart_builder("file", "video/mp4");
    let req = test::TestRequest::default()
        .uri("/video/temp/video")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body);

    let response = req.send_request(&app).await;
    assert_eq!(response.status(), StatusCode::OK);
    println!("test02 end");
}
