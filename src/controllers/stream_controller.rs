use crate::streamer::gstream_controller::create_streams;
use crate::streamer::types::{CompoundStreamInfo, StreamResolution, StreamStorage};
use actix_web::web::{Data, Path};
use actix_web::{post, web, HttpResponse, Responder, Scope};

pub fn register_scope() -> Scope {
    web::scope("/stream").service(start_stream)
}

#[post("/{video_id}")]
async fn start_stream(_path: Path<u32>, stream_storage: Data<StreamStorage>) -> impl Responder {
    // let video_id = path.into_inner();
    // TODO: handle database, permissions, dynamically render ID, etc...
    let main_stream = CompoundStreamInfo::new(
        String::from("1"),
        String::from("video_resources/video3.mp4"),
        vec![
            StreamResolution::P360,
            StreamResolution::P480,
            StreamResolution::P720,
        ],
    );

    match create_streams(stream_storage.clone(), main_stream).await {
        Ok(_) => {
            let streams = stream_storage.size();
            println!("{}", streams);
            HttpResponse::Ok()
        }
        Err(err) => {
            eprintln!("{}", err);
            HttpResponse::InternalServerError()
        }
    }
}
