use crate::streamer::gstreamer_controller::init_gstreamer;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::{info, warn};

use crate::api::controllers;
use crate::business::facades::artist::ArtistFacade;
use crate::business::facades::comment::CommentFacade;
use crate::api::routes::user::user_routes;
use crate::api::routes::video::video_routes;
use crate::business::facades::stream::StreamFacade;
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::facades::user::UserFacade;
use crate::business::facades::video::{VideoFacade, VideoFacadeTrait};
use crate::business::models::stream::StreamStorage;
use crate::configuration::models::Configuration;
use crate::persistence::repositories::artist::ArtistRepository;
use crate::persistence::repositories::comment::CommentRepository;
use crate::persistence::repositories::stream::PgStreamRepo;
use crate::persistence::repositories::temp_file::PgTempFileRepo;
use crate::persistence::repositories::user::PostgresUserRepo;
use crate::persistence::repositories::video::PgVideoRepo;
use config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;

mod api;
mod business;
mod configuration;
mod persistence;
mod streamer;

const CONFIG_FILE_KEY: &str = "CONFIG_FILE_PATH";

async fn setup_db_pool() -> anyhow::Result<Pool<Postgres>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
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

    let stream_storage = Arc::new(StreamStorage::new());
    let user_repo = Arc::new(PostgresUserRepo::new(pool.clone()));
    let user_facade = Arc::new(UserFacade::new(user_repo));

    let temp_file_repo = Arc::new(PgTempFileRepo::new(pool.clone()));
    let temp_file_facade = Arc::new(TempFileFacade::new(temp_file_repo));

    temp_file_facade
        .delete_all_temp_files()
        .await
        .expect("Failed to delete temp file directory");
    temp_file_facade
        .create_temp_directory()
        .await
        .expect("Failed to create temp directory");

    let video_repo = Arc::new(PgVideoRepo::new(pool.clone()));
    let video_facade = Arc::new(VideoFacade::new(temp_file_facade.clone(), video_repo));

    video_facade
        .create_dirs()
        .await
        .expect("Failed to create video folder");

    let stream_repo = Arc::new(PgStreamRepo::new(pool.clone()));
    let stream_facade = Arc::new(StreamFacade::new(
        video_facade.clone(),
        stream_storage.clone(),
        stream_repo.clone(),
    ));

    let artist_repo = ArtistRepository::new(pool.clone());
    let artist_facade = ArtistFacade::new(Arc::new(artist_repo));
    let comment_repo = CommentRepository::new(pool.clone());
    let comment_facade = CommentFacade::new(Arc::new(comment_repo));

    HttpServer::new(move || {
        App::new()
            .service(actix_files::Files::new("/static", "./static"))
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
            .app_data(web::Data::new(comment_facade.clone()))
            .app_data(web::Data::new(artist_facade.clone()))
            .service(controllers::user::list_users)
            .service(controllers::artist::list_artists)
            .service(controllers::comment::list_comments_to_video)
            .configure(video_routes)
            .configure(user_routes)
            .service(controllers::video::register_scope())
            .service(controllers::stream::register_scope())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;

    Ok(())
}

fn init_configuration() -> anyhow::Result<Configuration> {
    let config_file = dotenvy::var(CONFIG_FILE_KEY).unwrap_or(String::from("./config.yaml"));

    let config = Config::builder()
        .add_source(config::File::with_name(config_file.as_str()))
        .build()?;
    let config = config.try_deserialize::<Configuration>()?;

    info!("Config {} was loaded!", config_file);
    Ok(config)
}
