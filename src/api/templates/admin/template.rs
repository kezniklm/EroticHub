use askama::Template;

#[derive(Template)]
#[template(path = "admin/base.html")]
pub struct AdminSectionTemplate<T: Template> {
    child_template: T,
}

impl<T: Template> AdminSectionTemplate<T> {
    pub fn wrap(child_template: T) -> Self {
        Self { child_template }
    }
}
