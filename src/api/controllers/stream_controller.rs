use std::sync::Arc;
use crate::streamer::gstreamer_controller::create_streams;
use actix_web::web::{Data, Path};
use actix_web::{post, web, HttpResponse, Responder, Scope};
use log::error;
use crate::business::models::stream::{CompoundStreamInfo, StreamStorage};
use crate::streamer::types::StreamResolution;

pub fn register_scope() -> Scope {
    web::scope("/stream").service(start_stream)
}

#[post("/{video_id}")]
async fn start_stream(_path: Path<u32>, stream_storage: Data<StreamStorage>) -> impl Responder {
    // let video_id = path.into_inner();
    // TODO: handle database, permissions, dynamically render ID, move to business logic layer, etc...
    let main_stream = CompoundStreamInfo::new(
        String::from("1"),
        String::from("video_resources/video3.mp4"),
        vec![
            StreamResolution::P360,
            StreamResolution::P480,
            StreamResolution::P720,
        ],
    );
    match create_streams(stream_storage.into_inner(), Arc::new(main_stream)).await {
        Ok(_) => {
            HttpResponse::Ok()
        }
        Err(err) => {
            error!("{}", err);
            HttpResponse::InternalServerError()
        }
    }
}
