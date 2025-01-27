use crate::business::models::video::EditVideoTemplateModel;
use crate::business::models::video_category::VideoCategorySelected;
use askama_actix::Template;
#[allow(unused_imports)]
use crate::business::models::video::VideoVisibility; // used in video_edit.html template

#[derive(Template)]
#[template(path = "video/edit/video_edit.html")]
pub struct EditVideoTemplate<V: Template, T: Template> {
    pub video: EditVideoTemplateModel,
    pub video_input: V,
    pub thumbnail_input: T,
    pub categories: Vec<VideoCategorySelected>,
}
