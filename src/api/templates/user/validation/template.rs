use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/validation/index.html")]
pub struct ValidationTemplate {
    pub target_element: String,
    pub error_message: Option<String>,
}
