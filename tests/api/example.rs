use crate::common::files;
use actix_web::http::{Method, StatusCode};
use actix_web::{test, App};
use erotic_hub::common::tests::setup::AsyncContext;
use test_context::test_context;

// #[test_context(AsyncContext)]
// #[actix_web::test]
// async fn test01(ctx: &mut AsyncContext) {
//     let app = test::init_service(App::new().configure(ctx.configure_app())).await;
//     let req = test::TestRequest::default().uri("/user/login"); TODO FIX - identity problem
// 
//     let response = req.send_request(&app).await;
//     assert_eq!(response.status(), StatusCode::OK);
// }

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test02(ctx: &mut AsyncContext) {
    let app = test::init_service(App::new().configure(ctx.configure_app())).await;

    let (header, body) = files::VIDEO1.get_multipart_builder("file", "video/mp4");
    let req = test::TestRequest::default()
        .uri("/temp/video")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body);

    let response = req.send_request(&app).await;
    // println!("{:?}", response.into_body());
    assert_eq!(response.status(), StatusCode::CREATED);
}
