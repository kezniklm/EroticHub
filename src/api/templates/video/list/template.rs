use crate::business::models::video::VideoList;
use crate::business::models::video_category::VideoCategory;
use askama_actix::Template;

#[derive(Template)]
#[template(path = "video/list/index.html")]
pub struct IndexTemplate<T: Template> {
    pub videos_template: T,
    pub categories: Vec<VideoCategory>,
}

#[derive(Template)]
#[template(path = "video/list/video_grid.html")]
pub struct VideoGridTemplate {
    pub videos: Vec<VideoList>,
}

#[derive(Template)]
#[template(path = "video/list/videos.html")]
pub struct VideosTemplate {
    pub videos: Vec<VideoList>,
}
