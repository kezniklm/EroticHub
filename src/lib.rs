use crate::configuration::models::Configuration;
use actix_identity::IdentityMiddleware;
use actix_multipart::form::MultipartFormConfig;
use actix_session::config::PersistentSession;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::{Key, SameSite};
use actix_web::HttpResponse;
use config::Config;
use deadpool_redis::Runtime;
use log::info;
use std::env;
use std::sync::Arc;
use std::time::Duration;

pub mod api;
pub mod business;
pub mod common;
pub mod configuration;
pub mod persistence;
pub mod streamer;

pub const CONFIG_FILE_KEY: &str = "CONFIG_FILE_PATH";

const DEFAULT_VIDEO_DIRECTORY: &str = "./resources/videos";
const DEFAULT_THUMBNAILS_PATH: &str = "./resources/thumbnails";
const DEFAULT_PROFILE_PICTURE_DIRECTORY: &str = "./resources/images/users";
const VIDEOS_DIRECTORY_KEY: &str = "VIDEO_DIRECTORY_PATH";
const THUMBNAIL_DIRECTORY_KEY: &str = "THUMBNAIL_DIRECTORY_PATH";
const PROFILE_PICTURE_DIRECTORY_KEY: &str = "PROFILE_PICTURE_DIRECTORY_PATH";

const DEFAULT_TEMP_DIRECTORY: &str = "temp";
const TEMP_DIRECTORY_KEY: &str = "TEMP_DIRECTORY_PATH";

pub fn init_configuration() -> anyhow::Result<Configuration> {
    let config_file = dotenvy::var(CONFIG_FILE_KEY).unwrap_or(String::from("./config.yaml"));

    let config = Config::builder()
        .add_source(config::File::with_name(config_file.as_str()))
        .build()?;
    let config = config.try_deserialize::<Configuration>()?;

    info!("Config {} was loaded!", config_file);
    Ok(config)
}

/// Function returns path to both video and thumbnail folder, where the files are stored.
///
/// # Returns
///
/// Tuple with:
/// - Path to video directory as String
/// - Path to thumbnails directory as String
pub fn get_video_thumbnail_dirs() -> (String, String) {
    let video = dotenvy::var(VIDEOS_DIRECTORY_KEY).unwrap_or(DEFAULT_VIDEO_DIRECTORY.to_string());
    let thumbnail =
        dotenvy::var(THUMBNAIL_DIRECTORY_KEY).unwrap_or(DEFAULT_THUMBNAILS_PATH.to_string());
    (video, thumbnail)
}

pub fn get_temp_directory_path() -> String {
    dotenvy::var(TEMP_DIRECTORY_KEY).unwrap_or(DEFAULT_TEMP_DIRECTORY.to_string())
}

pub fn get_profile_picture_folder_path() -> String {
    dotenvy::var(PROFILE_PICTURE_DIRECTORY_KEY)
        .unwrap_or(DEFAULT_PROFILE_PICTURE_DIRECTORY.to_string())
}

pub fn get_secret_key() -> Key {
    let secret_key = match env::var("SESSION_SECRET_KEY") {
        Ok(secret_key) => secret_key,
        Err(_) => return Key::generate(),
    };

    if secret_key.len() < 64 {
        panic!("SESSION_SECRET_KEY must be at least 32 characters long");
    }

    Key::from(secret_key.as_bytes())
}

pub fn setup_auth(
    redis_store: &RedisSessionStore,
) -> (IdentityMiddleware, SessionMiddleware<RedisSessionStore>) {
    let cookie_expiration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days

    let identity_middleware = IdentityMiddleware::builder()
        .visit_deadline(Some(cookie_expiration))
        .build();

    let session_middleware = SessionMiddleware::builder(redis_store.clone(), get_secret_key())
        .cookie_name("erotic-hub".to_string())
        .cookie_secure(false) // Use secure cookies (only HTTPS)
        .cookie_http_only(true) // Prevent JavaScript access
        .cookie_same_site(SameSite::Lax) // Set SameSite policy
        .cookie_path("/".to_string()) // Set path for the cookie
        .session_lifecycle(
            PersistentSession::default().session_ttl(cookie_expiration.try_into().unwrap()),
        )
        .build();

    (identity_middleware, session_middleware)
}

pub async fn setup_redis_pool() -> anyhow::Result<deadpool_redis::Pool> {
    let redis_url = env::var("REDIS_DATABASE_URL").expect("REDIS_DATABASE_URL must be set");
    let redis_config = deadpool_redis::Config::from_url(&redis_url);
    let pool = redis_config.create_pool(Some(Runtime::Tokio1))?;

    Ok(pool)
}

pub fn setup_multipart_config(config: Arc<Configuration>) -> MultipartFormConfig {
    let total_limit_mb = config.app.file_size_limit_mb * 1024 * 1024;
    MultipartFormConfig::default()
        .total_limit(total_limit_mb as usize)
        .memory_limit(200 * 1024 * 1024)
        .error_handler(|err, _req| {
            let response = HttpResponse::BadRequest().force_close().finish();
            actix_web::error::InternalError::from_response(err, response).into()
        })
}
