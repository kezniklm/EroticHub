use askama_actix::Template;

#[derive(Template)]
#[template(path = "video/upload/index.html")]
pub struct VideoUploadTemplate {}
