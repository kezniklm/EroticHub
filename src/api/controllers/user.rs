use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::templates::template::BaseTemplate;
use crate::api::templates::user::delete::template::DeleteTemplate;
use crate::api::templates::user::detail::template::UserDetailTemplate;
use crate::api::templates::user::liked_videos::template::LikedVideosTemplate;
use crate::api::templates::user::logged_in::template::UserLoggedInTemplate;
use crate::api::templates::user::login::template::UserLoginTemplate;
use crate::api::templates::user::password_change::template::PasswordChangeTemplate;
use crate::api::templates::user::register::template::UserRegisterTemplate;
use crate::api::templates::user::validation::template::ValidationTemplate;
use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use crate::business::models::error::MapToAppError;
use crate::business::models::user::UserRole::{self, Registered};
use crate::business::models::user::{
    EmailQuery, ProfilePictureUpdate, UserDetailUpdate, UserLogin, UserPasswordUpdate,
    UserRegisterMultipart, UserSessionData, UsernameQuery,
};
use crate::business::Result;
use actix_identity::Identity;
use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use actix_web_grants::protect;
use askama_actix::TemplateToResponse;

pub async fn register_form(
    htmx_request: HtmxRequest,
    session: Session,
    identity: Option<Identity>,
) -> impl Responder {
    match identity {
        Some(_) => HttpResponse::SeeOther()
            .append_header(("Location", "/"))
            .finish(),
        None => BaseTemplate::wrap(htmx_request, session, UserRegisterTemplate {}).to_response(),
    }
}

pub async fn register_user(
    user_facade: web::Data<UserFacade>,
    session: Session,
    request: HttpRequest,
    identity: Option<Identity>,
    MultipartForm(user_register): MultipartForm<UserRegisterMultipart>,
) -> Result<impl Responder> {
    if identity.is_some() {
        return Ok(HttpResponse::SeeOther()
            .append_header(("Location", "/"))
            .finish());
    }

    let user = user_facade.register(user_register).await?;

    Identity::login(&request.extensions(), user.id.to_string())?;

    session.insert(
        "user_session_data",
        UserSessionData {
            profile_picture_path: user.profile_picture_path.clone(),
        },
    )?;

    Ok(UserLoggedInTemplate {
        profile_picture_path: user.profile_picture_path,
    }
    .to_response())
}

pub async fn login_form(
    htmx_request: HtmxRequest,
    session: Session,
    identity: Option<Identity>,
) -> impl Responder {
    match identity {
        Some(_) => HttpResponse::SeeOther()
            .append_header(("Location", "/"))
            .finish(),
        None => BaseTemplate::wrap(htmx_request, session, UserLoginTemplate {}).to_response(),
    }
}

pub async fn login(
    user_facade: web::Data<UserFacade>,
    session: Session,
    request: HttpRequest,
    identity: Option<Identity>,
    user_login: web::Form<UserLogin>,
) -> Result<impl Responder> {
    if identity.is_some() {
        return Ok(HttpResponse::SeeOther()
            .append_header(("Location", "/"))
            .finish());
    }

    let user = user_facade.login(user_login.into_inner()).await?;

    Identity::login(&request.extensions(), user.id.to_string())?;

    session.insert(
        "user_session_data",
        UserSessionData {
            profile_picture_path: user.profile_picture_path.clone(),
        },
    )?;

    Ok(UserLoggedInTemplate {
        profile_picture_path: user.profile_picture_path,
    }
    .to_response())
}

pub async fn user_detail(
    user_facade: web::Data<UserFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    identity: Identity,
) -> Result<impl Responder> {
    let user_session_data = session
        .get::<UserSessionData>("user_session_data")
        .unwrap_or(None);

    let user_detail = user_facade
        .get_user_detail(identity.id()?.parse().app_error("Unauthorised")?)
        .await?;

    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        UserDetailTemplate {
            user_session_data,
            user_detail,
        },
    )
    .to_response())
}

pub async fn user_update(
    user_facade: web::Data<UserFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    identity: Identity,
    user_detail_update: web::Form<UserDetailUpdate>,
) -> Result<impl Responder> {
    let user_detail = user_facade
        .update(
            identity.id()?.parse().app_error("Unauthorised")?,
            user_detail_update.into_inner(),
        )
        .await?;

    let user_session_data = UserSessionData {
        profile_picture_path: match user_detail.clone() {
            Some(user_detail) => user_detail.profile_picture_path,
            None => None,
        },
    };

    session.insert("user_session_data", user_session_data.clone())?;

    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        UserDetailTemplate {
            user_session_data: Some(user_session_data),
            user_detail,
        },
    )
    .to_response())
}

pub async fn change_password_form(
    htmx_request: HtmxRequest,
    session: Session,
    _identity: Identity,
) -> Result<impl Responder> {
    Ok(BaseTemplate::wrap(htmx_request, session, PasswordChangeTemplate {}).to_response())
}

pub async fn change_password(
    user_facade: web::Data<UserFacade>,
    htmx_request: HtmxRequest,
    session: Session,
    identity: Identity,
    user_password_update: web::Form<UserPasswordUpdate>,
) -> Result<impl Responder> {
    user_facade
        .change_password(
            identity.id()?.parse().app_error("Unauthorised")?,
            user_password_update.into_inner(),
        )
        .await?;

    let user_session_data = session
        .get::<UserSessionData>("user_session_data")
        .unwrap_or(None);

    let user_detail = user_facade
        .get_user_detail(identity.id()?.parse().app_error("Unauthorised")?)
        .await?;

    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        UserDetailTemplate {
            user_session_data,
            user_detail,
        },
    )
    .to_response())
}

pub async fn delete_form(
    htmx_request: HtmxRequest,
    session: Session,
    _identity: Identity,
) -> Result<impl Responder> {
    Ok(BaseTemplate::wrap(htmx_request, session, DeleteTemplate {}).to_response())
}

pub async fn delete(
    user_facade: web::Data<UserFacade>,
    identity: Identity,
) -> Result<impl Responder> {
    user_facade
        .delete_user(identity.id()?.parse().app_error("Unauthorised")?)
        .await?;

    identity.logout();

    Ok(HttpResponse::SeeOther()
        .append_header(("HX-Redirect", "/"))
        .finish())
}

pub async fn profile_picture_update(
    user_facade: web::Data<UserFacade>,
    session: Session,
    identity: Identity,
    MultipartForm(profile_picture_update): MultipartForm<ProfilePictureUpdate>,
) -> Result<impl Responder> {
    let user_detail = user_facade
        .update_profile_picture(
            identity.id()?.parse().app_error("Unauthorised")?,
            profile_picture_update,
        )
        .await?;

    let user_session_data = UserSessionData {
        profile_picture_path: match user_detail.clone() {
            Some(user_detail) => user_detail.profile_picture_path,
            None => None,
        },
    };

    session.insert("user_session_data", user_session_data.clone())?;

    Ok(HttpResponse::SeeOther()
        .append_header(("HX-Redirect", "/user/account"))
        .finish())
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn liked_videos(
    htmx_request: HtmxRequest,
    session: Session,
    identity: Identity,
) -> Result<impl Responder> {
    Ok(BaseTemplate::wrap(
        htmx_request,
        session,
        LikedVideosTemplate {
            user_id: identity.id()?,
        },
    )
    .to_response())
}

pub async fn validate_username(
    user_facade: web::Data<UserFacade>,
    username_query: web::Query<UsernameQuery>,
) -> impl Responder {
    let validation_result = user_facade
        .validate_username_exists(username_query.username.clone())
        .await;

    let validate_username_template = ValidationTemplate {
        error_message: validation_result.err().map(|error| error.code.to_string()),
        target_element: username_query.target_element.clone(),
    };

    validate_username_template.to_response()
}

pub async fn validate_email(
    user_facade: web::Data<UserFacade>,
    email_query: web::Query<EmailQuery>,
) -> impl Responder {
    let validation_result = user_facade
        .validate_email_exists(email_query.email.clone())
        .await;

    let validate_email_template = ValidationTemplate {
        error_message: validation_result.err().map(|error| error.code.to_string()),
        target_element: email_query.target_element.clone(),
    };

    validate_email_template.to_response()
}

pub async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}
