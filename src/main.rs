use actix_web::{web, App, HttpServer};
use log::{info, warn};

use crate::api::controllers;
use crate::business::facades::user::UserFacade;
use crate::configuration::models::Configuration;
use crate::persistence::repositories::user::UserRepository;
use config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;

mod api;
mod business;
mod common;
mod configuration;
mod persistence;

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

    let config = init_configuration().expect("Failed to load config.yaml");
    let pool = setup_db_pool().await?;

    let user_repo = UserRepository::new(pool.clone());

    let user_facade = UserFacade::new(Arc::new(user_repo));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(user_facade.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(controllers::user::list_users)
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
