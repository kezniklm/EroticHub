use crate::api::controllers::membership::{
    change_payment_method, get_deal_form, get_membership_details, get_payment_form,
    get_payment_method_form, pay,
};
use actix_web::web;

pub fn membership_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/membership")
            .route("", web::get().to(get_membership_details))
            .route("/payment-method", web::get().to(get_payment_method_form))
            .route("/payment-method", web::post().to(change_payment_method))
            .route("/deal", web::get().to(get_deal_form))
            .route("/deal/{deal_id}", web::get().to(get_payment_form))
            .route("/deal/{deal_id}", web::post().to(pay)),
    );
}
