use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/password_change/index.html")]
pub struct PasswordChangeTemplate {}
