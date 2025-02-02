use crate::common::users::{JOHN_ARTIST, JOHN_NOT_ARTIST};
use crate::common::utils::login_as;
use actix_multipart_test::MultiPartFormDataBuilder;
use actix_web::{http::StatusCode, test};
use erotic_hub::business::models::user::{UserDetailUpdate, UserPasswordUpdate};
use erotic_hub::common::tests::setup::AsyncContext;
use http::Method;
use test_context::test_context;

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test01_register_form_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::GET)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test02_register_form_already_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let cookie = login_as(JOHN_ARTIST, &app).await;

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::GET)
        .cookie(cookie)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(response.status().is_redirection());
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test03_register_user_ok(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let (header, body) = MultiPartFormDataBuilder::new()
        .with_text("username", "new_user123")
        .with_text("email", "new_user123@example.com")
        .with_text("password", "password12345")
        .with_text("password2", "password12345")
        .build();

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(response.status().is_success());
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test04_register_user_already_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let (header, body) = MultiPartFormDataBuilder::new()
        .with_text("username", "new_user123")
        .with_text("email", "new_user123@example.com")
        .with_text("password", "password12345")
        .with_text("password2", "password12345")
        .build();

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::POST)
        .cookie(cookie)
        .insert_header(header)
        .set_payload(body)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(response.status().is_redirection());
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test05_register_user_username_exists(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let existing_user = JOHN_ARTIST.get_login_req();

    let (header, body) = MultiPartFormDataBuilder::new()
        .with_text("username", existing_user.username)
        .with_text("email", "new_user123@example.com")
        .with_text("password", "password12345")
        .with_text("password2", "password12345")
        .build();

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test06_register_user_email_exists(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let existing_user_email = "john1@email.cz";

    let (header, body) = MultiPartFormDataBuilder::new()
        .with_text("username", "new_user")
        .with_text("email", existing_user_email)
        .with_text("password", "password12345")
        .with_text("password2", "password12345")
        .build();

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test07_register_user_passwords_not_equal(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let (header, body) = MultiPartFormDataBuilder::new()
        .with_text("username", "new_user")
        .with_text("email", "new_user123@example.com")
        .with_text("password", "password12345")
        .with_text("password2", "password123456")
        .build();

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test08_register_user_passwords_too_short(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let (header, body) = MultiPartFormDataBuilder::new()
        .with_text("username", "new_user")
        .with_text("email", "new_user123@example.com")
        .with_text("password", "passwor")
        .with_text("password2", "passwor")
        .build();

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test09_register_user_username_too_short(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let (header, body) = MultiPartFormDataBuilder::new()
        .with_text("username", "ne")
        .with_text("email", "new_user123@example.com")
        .with_text("password", "password")
        .with_text("password2", "password")
        .build();

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test10_register_user_incorrect_email(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let (header, body) = MultiPartFormDataBuilder::new()
        .with_text("username", "new_user")
        .with_text("email", "example.com")
        .with_text("password", "password")
        .with_text("password2", "password")
        .build();

    let request = test::TestRequest::default()
        .uri("/user/register")
        .method(Method::POST)
        .insert_header(header)
        .set_payload(body)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test11_login_form_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/login")
        .method(Method::GET)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test12_login_form_already_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let request = test::TestRequest::default()
        .uri("/user/login")
        .method(Method::GET)
        .cookie(cookie)
        .to_request();

    let response = test::call_service(&app, request).await;
    assert!(response.status().is_redirection());
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test13_login_ok(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let form_data = JOHN_ARTIST.get_login_req();

    let request = test::TestRequest::default()
        .uri("/user/login")
        .method(Method::POST)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(response.status().is_success());
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test14_login_wrong_username(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let mut form_data = JOHN_ARTIST.get_login_req();

    form_data.username = "wrongusername".to_string();

    let request = test::TestRequest::default()
        .uri("/user/login")
        .method(Method::POST)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should fail login"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test15_login_wrong_password(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let mut form_data = JOHN_ARTIST.get_login_req();

    form_data.password = "dunno12345".to_string();

    let request = test::TestRequest::default()
        .uri("/user/login")
        .method(Method::POST)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should fail login"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test16_user_detail_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/account")
        .method(Method::GET)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(response.status().is_redirection() || response.status() == StatusCode::UNAUTHORIZED);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test17_user_detail_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let request = test::TestRequest::default()
        .uri("/user/account")
        .method(Method::GET)
        .cookie(cookie)
        .to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test18_user_update_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let form_data = UserDetailUpdate {
        username: "NewUsername".to_string(),
        email: "usernam@gmail.com".to_string(),
    };

    let request = test::TestRequest::default()
        .uri("/user/account/edit")
        .method(Method::POST)
        .cookie(cookie)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test19_user_update_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let form_data = UserDetailUpdate {
        username: "NewUsername".to_string(),
        email: "usernam@gmail.com".to_string(),
    };

    let request = test::TestRequest::default()
        .uri("/user/account/edit")
        .method(Method::POST)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test20_user_update_username_exists(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let form_data = UserDetailUpdate {
        username: JOHN_NOT_ARTIST.get_login_req().username,
        email: "user@gmail.com".to_string(),
    };

    let request = test::TestRequest::default()
        .uri("/user/account/edit")
        .method(Method::POST)
        .cookie(cookie)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test21_user_update_email_exists(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let existing_email = "john2@email.cz";

    let form_data = UserDetailUpdate {
        username: "NewUsername".to_string(),
        email: existing_email.to_string(),
    };

    let request = test::TestRequest::default()
        .uri("/user/account/edit")
        .method(Method::POST)
        .cookie(cookie)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test22_change_password_form(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let request = test::TestRequest::default()
        .uri("/user/change-password")
        .cookie(cookie)
        .to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test23_change_password_form_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/change-password")
        .to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test24_change_password(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let form_data = UserPasswordUpdate {
        old_password: "12345678".to_string(),
        password: "tatra148".to_string(),
        password2: "tatra148".to_string(),
    };

    let request = test::TestRequest::default()
        .uri("/user/change-password")
        .method(Method::POST)
        .cookie(cookie)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(response.status().is_success());
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test25_change_password_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let form_data = UserPasswordUpdate {
        old_password: "12345678".to_string(),
        password: "tatra148".to_string(),
        password2: "tatra148".to_string(),
    };

    let request = test::TestRequest::default()
        .uri("/user/change-password")
        .method(Method::POST)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test26_change_password_passwords_not_equal(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let cookie = login_as(JOHN_ARTIST, &app).await;

    let form_data = UserPasswordUpdate {
        old_password: "12345678".to_string(),
        password: "tatra1489".to_string(),
        password2: "tatra148".to_string(),
    };

    let request = test::TestRequest::default()
        .uri("/user/change-password")
        .method(Method::POST)
        .cookie(cookie)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test27_change_password_passwords_too_short(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let cookie = login_as(JOHN_ARTIST, &app).await;

    let form_data = UserPasswordUpdate {
        old_password: "12345678".to_string(),
        password: "tatra".to_string(),
        password2: "tatra".to_string(),
    };

    let request = test::TestRequest::default()
        .uri("/user/change-password")
        .method(Method::POST)
        .cookie(cookie)
        .set_form(&form_data)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test28_delete_form(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let request = test::TestRequest::default()
        .uri("/user/delete")
        .cookie(cookie)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test29_delete_form_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/delete")
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test30_delete(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let cookie = login_as(JOHN_NOT_ARTIST, &app).await;

    let request = test::TestRequest::default()
        .uri("/user/delete")
        .method(Method::POST)
        .cookie(cookie)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(response.status().is_redirection());
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test31_delete_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/delete")
        .method(Method::POST)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test32_liked_videos_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/liked-videos")
        .method(Method::GET)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status().is_redirection(),
        "Non-registered user should not have access"
    );
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test33_liked_videos(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let cookie = login_as(JOHN_NOT_ARTIST, &app).await;

    let request = test::TestRequest::default()
        .uri("/user/likes")
        .method(Method::GET)
        .cookie(cookie)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test34_validate_username(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/validate/username?username=JohnArtist&target_element=username_field")
        .method(Method::GET)
        .to_request();

    let response = test::call_service(&app, request).await;

    let body = test::read_body(response).await;
    let body_str = String::from_utf8_lossy(&body);

    assert!(body_str.contains("already exists"));
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test35_validate_email(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/validate/email?email=john1@email.cz&target_element=email_field")
        .method(Method::GET)
        .to_request();

    let response = test::call_service(&app, request).await;

    let body = test::read_body(response).await;
    let body_str = String::from_utf8_lossy(&body);

    assert!(body_str.contains("already exists"));
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test36_logout(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;
    let cookie = login_as(JOHN_ARTIST, &app).await;

    let request = test::TestRequest::default()
        .uri("/user/logout")
        .method(Method::GET)
        .cookie(cookie)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert!(response.status().is_redirection());
}

#[test_context(AsyncContext)]
#[actix_web::test]
async fn test37_logout_not_logged_in(ctx: &mut AsyncContext) {
    let app = ctx.create_app().await;

    let request = test::TestRequest::default()
        .uri("/user/logout")
        .method(Method::GET)
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
