use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::warn;

use erotic_hub::api::routes::stream::stream_routes;
use erotic_hub::api::routes::temp_file::temp_file_routes;
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
use erotic_hub::streamer::gstreamer_controller::init_gstreamer;
use erotic_hub::{get_temp_directory_path, get_video_thumbnail_dirs, init_configuration};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;

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
            .configure(temp_file_routes)
            .configure(stream_routes)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;

    Ok(())
}
