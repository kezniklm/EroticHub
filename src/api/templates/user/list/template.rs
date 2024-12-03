use crate::business::models::user_detail::UserDetail;
use askama::Template;

#[derive(Template)]
#[template(path = "user/list/index.html")]
pub struct UserListTemplate {
    pub users: Vec<UserDetail>,
}
