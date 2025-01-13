use crate::business::models::video::EditVideoTemplateModel;
use askama_actix::Template;

#[derive(Template)]
#[template(path = "video/edit/video_edit.html")]
pub struct EditVideoTemplate<V: Template, T: Template> {
    pub video: EditVideoTemplateModel,
    pub video_input: V,
    pub thumbnail_input: T,
}
