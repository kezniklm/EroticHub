use crate::business::models::user::{UserDetail, UserSessionData};
use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/detail/index.html")]
pub struct UserDetailTemplate {
    pub user_session_data: Option<UserSessionData>,
    pub user_detail: Option<UserDetail>,
}
