use actix_web::web;

use crate::api::controllers::admin::{get_admin_memberships, get_admin_section};

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("", web::get().to(get_admin_section))
            .route("/memberships", web::get().to(get_admin_memberships)),
    );
}
