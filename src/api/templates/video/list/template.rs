use askama::Template;

#[derive(Template)]
#[template(path = "video/list/index.html")]
pub struct VideoListTemplate {}
