use askama_actix::Template;

use crate::business::models::video_category::VideoCategory;

#[derive(Template)]
#[template(path = "admin/categories/index.html")]
pub struct AdminCategoriesTemplate {
    pub categories: Vec<VideoCategory>,
}
