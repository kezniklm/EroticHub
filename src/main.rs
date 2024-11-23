use crate::streamer::types::{StreamInfo, StreamStorage};
use actix_web::{web, App, HttpServer};
use log::warn;

use crate::api::controllers;
use crate::business::facades::user::UserFacade;
use crate::persistence::repositories::user::PostgresUserRepo;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;

mod api;
mod business;
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

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        warn!("failed loading .env file: {e}")
    };

    init_gstream().expect("Failed to initialize GStreamer. Check if you have it installed on your system");

    let pool = setup_db_pool().await?;

    let stream_storage = StreamStorage::new();
    let user_repo = PostgresUserRepo::new(pool.clone());

    let user_facade = UserFacade::new(Arc::new(user_repo));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(stream_storage.clone()))
            .app_data(web::Data::new(user_repo.clone()))
            .app_data(web::Data::new(user_facade.clone()))
            .service(controllers::user::list_users)
            .service(controllers::stream_controller::register_scope())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;

    Ok(())
}
