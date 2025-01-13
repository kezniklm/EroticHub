use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use validator::Validate;

#[derive(Clone, Debug)]
pub struct UserDetail {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub profile_picture_path: Option<String>,
    pub artist_id: Option<i32>,
    pub paying_member_id: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct UserRegister {
    #[validate(length(min = 3, max = 12))]
    pub username: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub password2: String,
    #[validate(email)]
    pub email: String,
}

#[derive(MultipartForm)]
pub struct UserRegisterMultipart {
    pub username: Text<String>,
    pub password: Text<String>,
    pub password2: Text<String>,
    pub email: Text<String>,
    #[multipart(limit = "10MB")]
    pub profile_picture: Option<TempFile>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSessionData {
    pub profile_picture_path: Option<String>,
}

#[derive(Deserialize)]
pub struct UsernameQuery {
    pub username: String,
    pub target_element: String,
}

#[derive(Deserialize)]
pub struct EmailQuery {
    pub email: String,
    pub target_element: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum UserRole {
    PayingMember,
    Registered,
    Artist,
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}
