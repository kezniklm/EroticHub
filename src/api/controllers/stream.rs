use crate::business::facades::stream::{StreamFacade, StreamFacadeTrait};
use crate::business::models::stream::LiveStreamStart;
use crate::business::Result;
use actix_web::web::{Data, Form};
use actix_web::{HttpResponse, Responder};

pub async fn start_stream(
    Form(request): Form<LiveStreamStart>,
    stream_facade: Data<StreamFacade>,
) -> Result<impl Responder> {
    let stream_url = stream_facade.start_stream(request, 2).await?;

    Ok(HttpResponse::Created().body(stream_url))
}
