use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/detail/index.html")]
pub struct UserDetailTemplate {
    pub user_id: String, //TODO REMOVE - IT IS ONLY TEMPORARY
}
