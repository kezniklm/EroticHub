use crate::api::controllers::payment::change_payment_method;
use actix_web::web;

pub fn payment_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/payment").route("/{user_id}", web::post().to(change_payment_method)));
}
