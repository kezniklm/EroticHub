use crate::common::users::TestUser;
use actix_http::body::{BoxBody, EitherBody};
use actix_http::Request;
use actix_service::Service;
use actix_web::cookie::Cookie;
use actix_web::dev::ServiceResponse;
use actix_web::{test, Error};
use http::Method;
use std::str::FromStr;

pub async fn login_as(
    user: TestUser,
    app: &impl Service<Request, Response = ServiceResponse<EitherBody<BoxBody>>, Error = Error>,
) -> Cookie {
    let login_req = test::TestRequest::default()
        .uri("/user/login")
        .method(Method::POST)
        .set_form(user.get_login_req())
        .to_request();
    let login_resp = test::call_service(app, login_req).await;
    let cookie = login_resp
        .response()
        .cookies()
        .next()
        .expect("Missing cookie in login response");

    cookie.into_owned()
}

pub async fn extract_id(response: ServiceResponse<EitherBody<BoxBody>>) -> i32 {
    let temp_file_id_str = String::from_utf8(test::try_read_body(response).await.unwrap().to_vec())
        .expect("Expected string in response");

    temp_file_id_str
        .parse()
        .expect("Failed to parse response body to i32")
}

pub fn create_empty_cookie<'a>() -> Cookie<'a> {
    Cookie::from_str("erotic-hub=''").unwrap()
}
