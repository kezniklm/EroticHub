use crate::business::validation::regexes::RE_PATH;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct UserRegister {
    pub id: i32,
    pub username: String,
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub password2: String,
    #[validate(email)]
    pub email: String,
    #[validate(regex(path = *RE_PATH))]
    pub profile_picture_path: Option<String>,
}
