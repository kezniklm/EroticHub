use actix_web::web;

use crate::api::controllers::admin::{get_admin_deals, get_admin_section};

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("", web::get().to(get_admin_section))
            .route("/deals", web::get().to(get_admin_deals)),
    );
}
