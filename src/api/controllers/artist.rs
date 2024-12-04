use crate::business::facades::artist::{ArtistFacade, ArtistFacadeTrait};
use actix_web::{get, web, HttpResponse, Responder};

#[get("/")]
pub async fn list_artists(artist_facade: web::Data<ArtistFacade>) -> impl Responder {
    match artist_facade.list_artists().await {
        Ok(artists) => HttpResponse::Ok().json(artists),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
