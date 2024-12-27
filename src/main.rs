use crate::streamer::gstreamer_controller::init_gstreamer;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::{info, warn};

use erotic_hub::api::controllers;
use erotic_hub::api::routes::user::user_routes;
use erotic_hub::api::routes::video::video_routes;
use erotic_hub::business::facades::artist::ArtistFacade;
use erotic_hub::business::facades::comment::CommentFacade;
use erotic_hub::business::facades::stream::StreamFacade;
use erotic_hub::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use erotic_hub::business::facades::user::UserFacade;
use erotic_hub::business::facades::video::VideoFacade;
use erotic_hub::business::models::stream::StreamStorage;
use erotic_hub::persistence::repositories::artist::ArtistRepository;
use erotic_hub::persistence::repositories::comment::CommentRepository;
use erotic_hub::persistence::repositories::stream::PgStreamRepo;
use erotic_hub::persistence::repositories::temp_file::PgTempFileRepo;
use erotic_hub::persistence::repositories::user::PostgresUserRepo;
use erotic_hub::persistence::repositories::video::PgVideoRepo;
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

    let stream_storage = Arc::new(StreamStorage::default());
    let user_repo = Arc::new(PostgresUserRepo::new(pool.clone()));
    let user_facade = Arc::new(UserFacade::new(user_repo));

    let artist_repo = Arc::new(ArtistRepository::new(pool.clone()));
    let artist_facade = Arc::new(ArtistFacade::new(artist_repo));

    let comment_repo = Arc::new(CommentRepository::new(pool.clone()));
    let comment_facade = Arc::new(CommentFacade::new(comment_repo));

    let temp_file_repo = Arc::new(PgTempFileRepo::new(pool.clone()));
    let temp_file_facade = Arc::new(TempFileFacade::new(temp_file_repo));

    temp_file_facade
        .delete_all_temp_files()
        .await
        .expect("Failed to delete temp file directory");
    TempFileFacade::create_temp_directory()
        .await
        .expect("Failed to create temp directory");

    let video_repo = Arc::new(PgVideoRepo::new(pool.clone()));
    let video_facade = Arc::new(VideoFacade::new(temp_file_facade.clone(), video_repo));

    VideoFacade::create_dirs()
        .await
        .expect("Failed to create video folder");

    let stream_repo = Arc::new(PgStreamRepo::new(pool.clone()));
    let stream_facade = Arc::new(StreamFacade::new(
        video_facade.clone(),
        stream_storage.clone(),
        stream_repo.clone(),
    ));

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
            .app_data(web::Data::from(artist_facade.clone()))
            .app_data(web::Data::from(comment_facade.clone()))
            .configure(video_routes)
            .configure(user_routes)
            .configure(stream_routes)
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
