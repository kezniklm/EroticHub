use askama::Template;

#[derive(Template)]
#[template(path = "user/register/index.html")]
pub struct UserRegisterTemplate {}
