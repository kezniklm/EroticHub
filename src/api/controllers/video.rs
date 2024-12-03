use crate::api::templates::video::list::template::VideoListTemplate;
use actix_web::{HttpResponse, Responder};
use askama::Template;

pub async fn list_videos() -> impl Responder {
    let template = VideoListTemplate {};

    match template.render() {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
