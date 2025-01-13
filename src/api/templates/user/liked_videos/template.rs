use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/liked_videos/index.html")]
pub struct LikedVideosTemplate {
    pub user_id: String, //TODO REMOVE _ - IT IS ONLY TEMPORARY
}
