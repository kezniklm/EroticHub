use crate::api::extractors::permissions_extractor::extract;
use crate::api::routes::membership::membership_routes;
use crate::api::routes::stream::stream_routes;
use crate::api::routes::temp_file::temp_file_routes;
use crate::api::routes::user::user_routes;
use crate::api::routes::video::video_routes;
use crate::business::facades::artist::ArtistFacade;
use crate::business::facades::comment::CommentFacade;
use crate::business::facades::membership::MembershipFacade;
use crate::business::facades::stream::StreamFacade;
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::facades::user::UserFacade;
use crate::business::facades::video::VideoFacade;
use crate::business::models::stream::StreamStorage;
use crate::persistence::repositories::artist::ArtistRepository;
use crate::persistence::repositories::comment::CommentRepository;
use crate::persistence::repositories::paying_member::PostgresPayingMemberRepo;
use crate::persistence::repositories::payment_method::PostgresPaymentMethodRepo;
use crate::persistence::repositories::stream::PgStreamRepo;
use crate::persistence::repositories::temp_file::PgTempFileRepo;
use crate::persistence::repositories::user::UserRepository;
use crate::persistence::repositories::video::PgVideoRepo;
use actix_identity::IdentityMiddleware;
use actix_session::config::PersistentSession;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::{Key, SameSite};
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use deadpool_redis::Runtime;
use env_logger::Env;
use erotic_hub::streamer::gstreamer_controller::init_gstreamer;
use erotic_hub::{get_temp_directory_path, get_video_thumbnail_dirs, init_configuration};
use log::warn;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;
use std::time::Duration;

mod api;
mod business;
mod common;
mod configuration;
mod persistence;
mod streamer;

async fn setup_db_pool() -> anyhow::Result<Pool<Postgres>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

async fn setup_redis_pool() -> anyhow::Result<deadpool_redis::Pool> {
    let redis_url = env::var("REDIS_DATABASE_URL").expect("REDIS_DATABASE_URL must be set");
    let redis_config = deadpool_redis::Config::from_url(&redis_url);
    let pool = redis_config.create_pool(Some(Runtime::Tokio1))?;

    Ok(pool)
}

fn get_secret_key() -> Key {
    let secret_key = match env::var("SESSION_SECRET_KEY") {
        Ok(secret_key) => secret_key,
        Err(_) => return Key::generate(),
    };

    if secret_key.len() < 64 {
        panic!("SESSION_SECRET_KEY must be at least 32 characters long");
    }

    Key::from(secret_key.as_bytes())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        warn!("failed loading .env file: {e}")
    };

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    init_gstreamer()
        .expect("Failed to initialize GStreamer. Check if you have it installed on your system");
    let config = init_configuration().expect("Failed to load config.yaml");
    init_gstreamer()
        .expect("Failed to initialize GStreamer. Check if you have it installed on your system");

    let pool = setup_db_pool().await?;

    let redis_pool = setup_redis_pool().await?;

    let redis_store = RedisSessionStore::new_pooled(redis_pool).await?;

    let stream_storage = Arc::new(StreamStorage::default());
    let user_repo = Arc::new(UserRepository::new(pool.clone()));
    let user_facade = Arc::new(UserFacade::new(user_repo));

    let artist_repo = Arc::new(ArtistRepository::new(pool.clone()));
    let artist_facade = Arc::new(ArtistFacade::new(artist_repo));

    let comment_repo = Arc::new(CommentRepository::new(pool.clone()));
    let comment_facade = Arc::new(CommentFacade::new(comment_repo));

    let temp_file_repo = Arc::new(PgTempFileRepo::new(pool.clone()));

    let temp_dir_path = get_temp_directory_path();
    let temp_file_facade = Arc::new(TempFileFacade::new(temp_file_repo, temp_dir_path.clone()));

    temp_file_facade
        .delete_all_temp_files()
        .await
        .expect("Failed to delete temp file directory");
    TempFileFacade::create_temp_directory(temp_dir_path)
        .await
        .expect("Failed to create temp directory");

    let video_repo = Arc::new(PgVideoRepo::new(pool.clone()));
    let (video_dir, thumbnail_dir) = get_video_thumbnail_dirs();
    let video_facade = Arc::new(VideoFacade::new(
        temp_file_facade.clone(),
        video_repo,
        video_dir.clone(),
        thumbnail_dir.clone(),
    ));

    VideoFacade::create_dirs(video_dir, thumbnail_dir)
        .await
        .expect("Failed to create video folder");

    let stream_repo = Arc::new(PgStreamRepo::new(pool.clone()));
    let stream_facade = Arc::new(StreamFacade::new(
        video_facade.clone(),
        stream_storage.clone(),
        stream_repo.clone(),
    ));

    let paying_member_repo = Arc::new(PostgresPayingMemberRepo::new(pool.clone()));
    let payment_method_repo = Arc::new(PostgresPaymentMethodRepo::new(pool.clone()));
    let membership_facade = Arc::new(MembershipFacade::new(
        paying_member_repo,
        payment_method_repo,
    ));

    let secret_key = get_secret_key();

    HttpServer::new(move || {
        let cookie_expiration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days

        let identity_middleware = IdentityMiddleware::builder()
            .visit_deadline(Some(cookie_expiration))
            .build();

        let session_middleware =
            SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                .cookie_name("erotic-hub".to_string())
                .cookie_secure(false) // Use secure cookies (only HTTPS)
                .cookie_http_only(true) // Prevent JavaScript access
                .cookie_same_site(SameSite::Lax) // Set SameSite policy
                .cookie_path("/".to_string()) // Set path for the cookie
                .session_lifecycle(
                    PersistentSession::default().session_ttl(cookie_expiration.try_into().unwrap()),
                )
                .build();

        App::new()
            .service(actix_files::Files::new("/static", "./static"))
            .wrap(GrantsMiddleware::with_extractor(extract))
            .wrap(identity_middleware)
            .wrap(session_middleware)
            .wrap(NormalizePath::trim())
            .wrap(Logger::default())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::from(stream_storage.clone()))
            .app_data(web::Data::from(stream_facade.clone()))
            .app_data(web::Data::from(stream_storage.clone()))
            .app_data(web::Data::from(user_facade.clone()))
            .app_data(web::Data::from(temp_file_facade.clone()))
            .app_data(web::Data::from(user_facade.clone()))
            .app_data(web::Data::from(temp_file_facade.clone()))
            .app_data(web::Data::from(video_facade.clone()))
            .app_data(web::Data::from(artist_facade.clone()))
            .app_data(web::Data::from(comment_facade.clone()))
            .app_data(web::Data::from(membership_facade.clone()))
            .configure(video_routes)
            .configure(user_routes)
            .configure(temp_file_routes)
            .configure(stream_routes)
            .configure(membership_routes)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;

    Ok(())
}
