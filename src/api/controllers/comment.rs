use crate::business::facades::comment::{CommentFacade, CommentFacadeTrait};
use actix_web::{get, web, HttpResponse, Responder};

#[get("/comment/{video_id}")]
pub async fn list_comments_to_video(
    comment_facade: web::Data<CommentFacade>,
    video_id: web::Path<i32>,
) -> impl Responder {
    match comment_facade
        .list_comments_to_video(video_id.into_inner())
        .await
    {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
