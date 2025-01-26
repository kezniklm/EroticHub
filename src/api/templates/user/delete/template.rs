use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/delete/index.html")]
pub struct DeleteTemplate {}
