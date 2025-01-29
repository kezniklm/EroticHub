use actix_web::web;

use crate::api::controllers::admin::{
    add_deal, delete_deal, edit_deal, get_admin_add_deal_form, get_admin_deals,
    get_admin_edit_deal_form, get_admin_section, get_users, make_user_admin, make_user_artist,
    revoke_user_admin,
};

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("", web::get().to(get_admin_section))
            .route("/deals", web::get().to(get_admin_deals))
            .route("/deals/new", web::get().to(get_admin_add_deal_form))
            .route("/deals", web::post().to(add_deal))
            .route("/deals/{deal_id}", web::get().to(get_admin_edit_deal_form))
            .route("/deals/{deal_id}", web::put().to(edit_deal))
            .route("/deals/{deal_id}", web::delete().to(delete_deal))
            .route("/users", web::get().to(get_users))
            .route(
                "/users/{user_id}/make-artist",
                web::post().to(make_user_artist),
            )
            .route(
                "/users/{user_id}/make-admin",
                web::post().to(make_user_admin),
            )
            .route(
                "/users/{user_id}/revoke-admin",
                web::post().to(revoke_user_admin),
            ),
    );
}
