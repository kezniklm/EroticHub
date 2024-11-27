use crate::streamer::gstreamer_controller::init_gstreamer;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::warn;
use log::{info, warn};

use crate::api::controllers;
use crate::business::facades::user::UserFacade;
use crate::business::models::stream::StreamStorage;
use crate::configuration::models::Configuration;
use crate::persistence::repositories::user::PostgresUserRepo;
use config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::persistence::repositories::temp_file::PgTempFileRepo;

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
    init_gstream().expect("Failed to initialize GStreamer. Check if you have it installed on your system");

    let pool = setup_db_pool().await?;

    let stream_storage = StreamStorage::new();
    let user_repo = PostgresUserRepo::new(pool.clone());
    let user_facade = UserFacade::new(Arc::new(user_repo));

    let temp_file_repo = PgTempFileRepo::new(pool.clone());
    let temp_file_facade = TempFileFacade::new(Arc::new(temp_file_repo));
    temp_file_facade.create_temp_directory().await.expect("Failed to create temp directory");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(stream_storage.clone()))
            .app_data(web::Data::new(user_facade.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(temp_file_facade.clone()))
            .service(controllers::user::list_users)
            .service(controllers::video::register_scope())
            .service(controllers::stream_controller::register_scope())
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
