use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/logged_in/index.html")]
pub struct UserLoggedInTemplate {
    pub profile_picture_path: Option<String>,
}
