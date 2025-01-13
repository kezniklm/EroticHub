use crate::api::controllers::membership::{
    change_payment_method, get_membership_details, get_payment_method_form,
};
use actix_web::web;

pub fn membership_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/membership/{user_id}")
            .route("", web::get().to(get_membership_details))
            .route("/payment-method", web::get().to(get_payment_method_form))
            .route("/payment-method", web::post().to(change_payment_method)),
        // .route("/payment", web::get().to(get_payment_form)),
    );
}
