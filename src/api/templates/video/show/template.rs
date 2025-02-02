use crate::api::controllers::utils::route_util::{build_get_temp_path, build_get_video_path};
use crate::api::extractors::permissions_extractor::IsRole;
use crate::business::models::comment::CommentUserModel;
use crate::business::models::video::Video;
use actix_session::Session;
use askama_actix::Template;

#[derive(Template)]
#[template(path = "video/show/video_show.html")]
pub struct ShowVideoTemplate<T: Template> {
    pub video: Video,
    pub player_template: T,
    pub session: Session,
    pub user_id: i32,
    pub is_liked: bool,
    pub is_video_owner: bool,
}

#[derive(Template)]
#[template(path = "video/show/player.html")]
pub struct PlayerTemplate {
    video_path: String,
    thumbnail_path: Option<String>,
}

impl PlayerTemplate {
    pub fn from_saved(video_id: i32) -> Self {
        let (video_path, thumbnail_path) = build_get_video_path(video_id);
        Self {
            video_path,
            thumbnail_path: Some(thumbnail_path),
        }
    }

    pub fn from_temporary(temp_id: i32) -> Self {
        let video_path = build_get_temp_path(temp_id);
        Self {
            video_path,
            thumbnail_path: None,
        }
    }
}

#[derive(Template)]
#[template(path = "video/show/comments.html")]
pub struct CommentsTemplate {
    pub comments: Vec<CommentUserModel>,
}
