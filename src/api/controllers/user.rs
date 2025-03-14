use crate::api::controllers::utils::video_utils::from_video_to_video_list;
use crate::api::extractors::htmx_extractor::HtmxRequest;
use crate::api::extractors::permissions_extractor::AsInteger;
use crate::api::templates::template::BaseTemplate;
use crate::api::templates::user::delete::template::DeleteTemplate;
use crate::api::templates::user::detail::template::UserDetailTemplate;
use crate::api::templates::user::liked_videos::template::LikedVideosTemplate;
use crate::api::templates::user::logged_in::template::UserLoggedInTemplate;
use crate::api::templates::user::login::template::UserLoginTemplate;
use crate::api::templates::user::password_change::template::PasswordChangeTemplate;
use crate::api::templates::user::register::template::UserRegisterTemplate;
use crate::api::templates::user::validation::template::ValidationTemplate;
use crate::api::templates::video::list::template::VideosTemplate;
use crate::business::facades::artist::ArtistFacade;
use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use crate::business::facades::video::{VideoFacade, VideoFacadeTrait};
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
// TODO: after logging in or registering, a page reload is required to update the profile in navbar
// TODO: liked videos
// TODO: terms and conditions
// TODO: banners
// TODO: Create your own playlists., Engage with the community., and Tailored video suggestions. don't make sense
// TODO: become an artist

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

            roles: user_facade.get_permissions(user.id).await?,
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

            roles: user_facade.get_permissions(user.id).await?,
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

    let user_detail = user_facade.get_user_detail(identity.id_i32()?).await?;

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
        .update(identity.id_i32()?, user_detail_update.into_inner())
        .await?;

    let user_session_data = UserSessionData {
        profile_picture_path: match user_detail.clone() {
            Some(user_detail) => user_detail.profile_picture_path,
            None => None,
        },
        roles: user_facade.get_permissions(identity.id_i32()?).await?,
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
        .change_password(identity.id_i32()?, user_password_update.into_inner())
        .await?;

    let user_session_data = session
        .get::<UserSessionData>("user_session_data")
        .unwrap_or(None);

    let user_detail = user_facade.get_user_detail(identity.id_i32()?).await?;

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
    user_facade.delete_user(identity.id_i32()?).await?;

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
        .update_profile_picture(identity.id_i32()?, profile_picture_update)
        .await?;

    let user_session_data = UserSessionData {
        profile_picture_path: match user_detail.clone() {
            Some(user_detail) => user_detail.profile_picture_path,
            None => None,
        },
        roles: user_facade.get_permissions(identity.id_i32()?).await?,
    };

    session.insert("user_session_data", user_session_data.clone())?;

    Ok(HttpResponse::SeeOther()
        .append_header(("HX-Redirect", "/user/account"))
        .finish())
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn liked_videos(
    identity: Identity,
    user_facade: web::Data<UserFacade>,
    video_facade: web::Data<VideoFacade>,
    artist_facade: web::Data<ArtistFacade>,
) -> Result<impl Responder> {
    let user_id = identity.id_i32()?;
    let likes = user_facade.liked_videos_by_user(user_id).await?;

    let mut ids = vec![];
    for like in likes {
        ids.push(like.video_id);
    }

    let videos = video_facade.fetch_liked_videos(ids).await?;
    let serialized_videos = from_video_to_video_list(videos, artist_facade).await?;
    let template = VideosTemplate {
        videos: serialized_videos,
    };

    Ok(template.to_response())
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn likes_page(
    htmx_request: HtmxRequest,
    session: Session,
    _identity: Identity,
) -> Result<impl Responder> {
    Ok(BaseTemplate::wrap(htmx_request, session, LikedVideosTemplate {}).to_response())
}

#[protect(any("Registered"), ty = "UserRole")]
pub async fn like_video(
    user_facade: web::Data<UserFacade>,
    identity: Identity,
    video_id: web::Path<i32>,
) -> Result<impl Responder> {
    let user_id = identity.id_i32()?;
    match user_facade.is_liked_already(user_id, *video_id).await? {
        true => {
            user_facade
                .unlike_video(user_id, video_id.into_inner())
                .await?;
        }
        false => {
            user_facade
                .like_video(user_id, video_id.into_inner())
                .await?;
        }
    }

    Ok(HttpResponse::Ok()
        .append_header(("HX-Refresh", "true"))
        .finish())
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
