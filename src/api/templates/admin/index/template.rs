use askama_actix::Template;

#[derive(Template)]
#[template(path = "admin/index/index.html")]
pub struct AdminIndexTemplate {}
