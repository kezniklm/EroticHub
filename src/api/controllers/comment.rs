use crate::api::controllers::utils::comment_utils::format_datetime;
use crate::api::controllers::utils::route_util::add_redirect_header;
use crate::api::templates::video::show::template::CommentsTemplate;
use crate::business::facades::comment::{CommentFacade, CommentFacadeTrait};
use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use crate::business::models::comment::{CommentCreateModel, CommentUserModel, FetchCommentsOffset};
use actix_identity::Identity;
use actix_web::web::Query;
use actix_web::{web, HttpResponse, Responder, Result};
use askama_actix::TemplateToResponse;

pub async fn get_comments_to_video(
    comment_facade: web::Data<CommentFacade>,
    user_facade: web::Data<UserFacade>,
    req: Query<FetchCommentsOffset>,
    video_id: web::Path<i32>,
) -> Result<impl Responder> {
    let comments = comment_facade
        .list_comments_to_video(video_id.into_inner(), req.offset)
        .await?;

    let mut comments_with_users: Vec<CommentUserModel> = vec![];

    for comment in comments {
        let user = user_facade.get_user_detail(comment.user_id).await?;
        if user.is_none() {
            continue;
        };
        let user = user.unwrap();
        comments_with_users.push(CommentUserModel {
            id: comment.id,
            user_id: comment.user_id,
            comment_content: comment.content,
            created_at: format_datetime(comment.created_at),
            profile_picture_path: user.profile_picture_path,
            username: user.username,
        });
    }

    Ok(CommentsTemplate {
        comments: comments_with_users,
    }
    .to_response())
}

pub async fn create_comment(
    comment_facade: web::Data<CommentFacade>,
    form: web::Form<CommentCreateModel>,
    _identity: Identity,
) -> Result<impl Responder> {
    let video_id = form.video_id;
    comment_facade
        .create_comment_to_video(form.into_inner())
        .await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header(format!("/video/{}/watch", video_id).as_str(), &mut response)?;
    Ok(response)
}
