use crate::api::controllers::utils::route_util::add_redirect_header;
use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::admin::categories::template::AdminCategoriesTemplate;
use crate::api::templates::admin::deals::template::AdminDealsTemplate;
use crate::api::templates::admin::edit_deal::template::AdminEditDealTemplate;
use crate::api::templates::admin::index::template::AdminIndexTemplate;
use crate::api::templates::admin::template::AdminSectionTemplate;
use crate::api::templates::admin::users::template::AdminUsersTemplate;
use crate::api::templates::template::BaseTemplate;
use crate::business::facades::artist::{ArtistFacade, ArtistFacadeTrait};
use crate::business::facades::membership::{DealInput, MembershipFacade, MembershipFacadeTrait};
use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use crate::business::facades::video_category::{VideoCategoryFacade, VideoCategoryFacadeTrait};
use crate::business::models::error::AppError;
use crate::business::models::user::UserRole::{self, Admin};

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder, Result};
use actix_web_grants::protect;

#[protect(any("Admin"), ty = "UserRole")]
pub async fn get_admin_section(
    htmx_request: HtmxRequest,
    session: Session,
) -> Result<impl Responder> {
    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        AdminSectionTemplate::wrap(AdminIndexTemplate {}),
    ))
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn get_admin_deals(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
) -> Result<impl Responder> {
    let deals = membership_facade.get_deals().await?;

    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        AdminSectionTemplate::wrap(AdminDealsTemplate { deals }),
    ))
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn get_admin_edit_deal_form(
    membership_facade: web::Data<MembershipFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    deal_id: web::Path<i32>,
) -> Result<impl Responder> {
    let deal = membership_facade.get_deal(*deal_id).await?.ok_or_else(|| {
        AppError::new(
            "Deal not found",
            crate::business::models::error::AppErrorKind::NotFound,
        )
    })?;

    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        AdminSectionTemplate::wrap(AdminEditDealTemplate { deal: Some(deal) }),
    ))
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn edit_deal(
    membership_facade: web::Data<MembershipFacade>,
    edit_deal_input: web::Form<DealInput>,
    deal_id: web::Path<i32>,
) -> Result<impl Responder> {
    membership_facade
        .edit_deal(*deal_id, edit_deal_input.into_inner())
        .await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/admin/deals", &mut response)?;
    Ok(response)
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn delete_deal(
    membership_facade: web::Data<MembershipFacade>,
    deal_id: web::Path<i32>,
) -> Result<impl Responder> {
    membership_facade.delete_deal(*deal_id).await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/admin/deals", &mut response)?;
    Ok(response)
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn get_admin_add_deal_form(
    htmx_request: HtmxRequest,
    session: Session,
) -> Result<impl Responder> {
    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        AdminSectionTemplate::wrap(AdminEditDealTemplate { deal: None }),
    ))
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn add_deal(
    membership_facade: web::Data<MembershipFacade>,
    edit_deal_input: web::Form<DealInput>,
) -> Result<impl Responder> {
    membership_facade
        .add_deal(edit_deal_input.into_inner())
        .await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/admin/deals", &mut response)?;
    Ok(response)
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn get_users(
    user_facade: web::Data<UserFacade>,
    htmx_request: HtmxRequest,
    session: Session,
) -> Result<impl Responder> {
    let users = user_facade.get_users().await?;

    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        AdminSectionTemplate::wrap(AdminUsersTemplate { users }),
    ))
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn make_user_artist(
    artist_facade: web::Data<ArtistFacade>,
    user_id: web::Path<i32>,
) -> Result<impl Responder> {
    artist_facade.make_user_artist(*user_id).await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/admin/users", &mut response)?;
    Ok(response)
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn make_user_admin(
    user_facade: web::Data<UserFacade>,
    user_id: web::Path<i32>,
) -> Result<impl Responder> {
    user_facade.change_admin_status(*user_id, true).await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/admin/users", &mut response)?;
    Ok(response)
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn revoke_user_admin(
    user_facade: web::Data<UserFacade>,
    user_id: web::Path<i32>,
) -> Result<impl Responder> {
    user_facade.change_admin_status(*user_id, false).await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/admin/users", &mut response)?;
    Ok(response)
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn get_admin_categories(
    category_facade: web::Data<VideoCategoryFacade>,
    htmx_request: HtmxRequest,
    session: Session,
) -> Result<impl Responder> {
    let categories = category_facade.list_categories().await?;

    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        AdminSectionTemplate::wrap(AdminCategoriesTemplate { categories }),
    ))
}

#[derive(serde::Deserialize)]
pub struct CategoryForm {
    name: String,
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn add_category(
    category_facade: web::Data<VideoCategoryFacade>,
    form: web::Form<CategoryForm>,
) -> Result<impl Responder> {
    category_facade.add_category(form.into_inner().name).await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/admin/categories", &mut response)?;
    Ok(response)
}

#[protect(any("Admin"), ty = "UserRole")]
pub async fn delete_category(
    category_facade: web::Data<VideoCategoryFacade>,
    category_id: web::Path<i32>,
) -> Result<impl Responder> {
    category_facade.delete_category(*category_id).await?;

    let mut response = HttpResponse::NoContent().finish();
    add_redirect_header("/admin/categories", &mut response)?;
    Ok(response)
}
