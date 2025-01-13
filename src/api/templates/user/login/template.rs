use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/login/index.html")]
pub struct UserLoginTemplate {}
