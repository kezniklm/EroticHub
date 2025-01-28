use actix_web::web;

use crate::api::controllers::admin::{
    edit_deal, get_admin_deals, get_admin_edit_deal_form, get_admin_section,
};

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("", web::get().to(get_admin_section))
            .route("/deals", web::get().to(get_admin_deals))
            .route("/deals/{deal_id}", web::get().to(get_admin_edit_deal_form))
            .route("/deals/{deal_id}", web::put().to(edit_deal)),
    );
}
