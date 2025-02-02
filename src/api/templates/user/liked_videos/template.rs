use askama_actix::Template;

#[derive(Template)]
#[template(path = "user/liked_videos/index.html")]
pub struct LikedVideosTemplate {}
