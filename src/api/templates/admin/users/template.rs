use askama_actix::Template;

use crate::business::models::user::UserDetail;

#[derive(Template)]
#[template(path = "admin/users/index.html")]
pub struct AdminUsersTemplate {
    pub users: Vec<UserDetail>,
}
