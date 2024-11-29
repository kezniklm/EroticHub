use crate::business::facades::stream::{StreamFacade, StreamFacadeTrait};
use crate::business::models::stream::LiveStreamStart;
use actix_web::web::{Data, Form};
use actix_web::{post, web, HttpResponse, Responder, Scope};
use log::error;

pub fn register_scope() -> Scope {
    web::scope("/stream").service(start_stream)
}

#[post("/start")]
async fn start_stream(Form(request): Form<LiveStreamStart>, stream_facade: Data<StreamFacade>) -> impl Responder {
    match stream_facade.start_stream(request, 2).await {
        Ok(stream_url) => {
            HttpResponse::Ok().body(stream_url)
        }
        Err(err) => {
            error!("Failed to start the stream {:#?}", err);
            HttpResponse::InternalServerError().body("Failed to start the stream!")
        }
    }
}
