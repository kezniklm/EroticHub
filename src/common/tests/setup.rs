use crate::api::controllers;
use crate::api::routes::stream::stream_routes;
use crate::api::routes::user::user_routes;
use crate::api::routes::video::video_routes;
use crate::business::facades::artist::ArtistFacade;
use crate::business::facades::comment::CommentFacade;
use crate::business::facades::stream::StreamFacade;
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::facades::user::UserFacade;
use crate::business::facades::video::VideoFacade;
use crate::business::models::stream::StreamStorage;
use crate::init_configuration;
use crate::persistence::repositories::artist::ArtistRepository;
use crate::persistence::repositories::comment::CommentRepository;
use crate::persistence::repositories::stream::PgStreamRepo;
use crate::persistence::repositories::temp_file::PgTempFileRepo;
use crate::persistence::repositories::user::PostgresUserRepo;
use crate::persistence::repositories::video::PgVideoRepo;
use actix_web::web;
use actix_web::web::ServiceConfig;
use log::warn;
use sqlx::{Executor, PgPool};
use std::env;
use std::sync::Arc;
use test_context::AsyncTestContext;
use uuid::Uuid;

/// A context for managing an asynchronous test database lifecycle.
/// Template data are loaded into the test database
///
/// This struct is used with the `test_context` crate to manage a PostgreSQL
/// test database during tests.
/// To use the AsyncContext use #[test_context(AsyncContext)] along with #[tokio::test]
pub struct AsyncContext {
    pub pg_pool: PgPool,
    pub test_db_name: String,
    pub test_folders_root: String,
}

/// A context for managing an asynchronous test database lifecycle.
/// Template data are not loaded into the test database, so it's empty.
///
/// This struct is used with the `test_context` crate to manage a PostgreSQL
/// test database during tests.
/// To use the AsyncContext use #[test_context(EmptyAsyncContext)] along with #[tokio::test]
pub struct EmptyAsyncContext {
    pub pg_pool: PgPool,
    pub test_db_name: String,
    pub test_folders_root: String,
}

impl AsyncContext {
    pub fn configure_app(&self) -> impl Fn(&mut ServiceConfig) {
        let app_config = Arc::new(init_configuration().expect("Failed to load config.yaml"));
        let stream_storage = Arc::new(StreamStorage::default());
        let user_repo = Arc::new(PostgresUserRepo::new(self.pg_pool.clone()));
        let user_facade = Arc::new(UserFacade::new(user_repo));

        let artist_repo = Arc::new(ArtistRepository::new(self.pg_pool.clone()));
        let artist_facade = Arc::new(ArtistFacade::new(artist_repo));

        let comment_repo = Arc::new(CommentRepository::new(self.pg_pool.clone()));
        let comment_facade = Arc::new(CommentFacade::new(comment_repo));

        let temp_file_repo = Arc::new(PgTempFileRepo::new(self.pg_pool.clone()));
        let temp_file_facade = Arc::new(TempFileFacade::new(temp_file_repo));

        let video_repo = Arc::new(PgVideoRepo::new(self.pg_pool.clone()));
        let video_facade = Arc::new(VideoFacade::new(temp_file_facade.clone(), video_repo));

        let stream_repo = Arc::new(PgStreamRepo::new(self.pg_pool.clone()));
        let stream_facade = Arc::new(StreamFacade::new(
            video_facade.clone(),
            stream_storage.clone(),
            stream_repo.clone(),
        ));

        move |config: &mut ServiceConfig| {
            config
                .service(actix_files::Files::new("/static", "./static"))
                .app_data(web::Data::from(app_config.clone()))
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
                .service(controllers::video::register_scope());
        }
    }
}

impl AsyncTestContext for AsyncContext {
    /// Sets up the test database and returns the context
    ///
    /// This method is automatically called before tests that use this context.
    /// It creates a new test database and initializes a connection pool.
    async fn setup() -> AsyncContext {
        let test_id = Uuid::new_v4();
        let (pg_pool, test_db_name) = setup_test_db(test_id).await;
        load_template_data(&pg_pool).await;
        let test_folders_root = create_test_resources_dir(test_id).await;
        AsyncContext {
            pg_pool,
            test_db_name,
            test_folders_root,
        }
    }

    async fn teardown(self) {
        teardown(self.test_folders_root, self.pg_pool, &self.test_db_name).await;
    }
}

impl AsyncTestContext for EmptyAsyncContext {
    /// Sets up the test database and returns the context
    ///
    /// This method is automatically called before tests that use this context.
    /// It creates a new test database and initializes a connection pool.
    async fn setup() -> EmptyAsyncContext {
        let test_id = Uuid::new_v4();
        let (pg_pool, test_db_name) = setup_test_db(test_id).await;
        let test_folders_root = create_test_resources_dir(test_id).await;
        EmptyAsyncContext {
            pg_pool,
            test_db_name,
            test_folders_root,
        }
    }

    async fn teardown(self) {
        teardown(self.test_folders_root, self.pg_pool, &self.test_db_name).await;
    }
}

/// Tears down the test database
///
/// This method is automatically called after tests that use this context.
/// It drops the test database and cleans up resources.
async fn teardown(test_folders_root: String, pg_pool: PgPool, test_db_name: &str) {
    delete_test_resources_dir(test_folders_root).await;
    teardown_test_db(pg_pool, test_db_name).await;
}

/// Loads environment variables from a `.env` file
///
/// If the `.env` file cannot be found, a warning is logged.
fn load_env() {
    if let Err(e) = dotenvy::dotenv() {
        warn!("failed loading .env file: {e}")
    };
}

/// Replaces the database name in a given database URL
///
/// This function is used to generate a connection URL for a specific
/// database by replacing the placeholder `test_template_db` with the
/// provided `new_db_name`.
///
/// # Arguments
///
/// * `db_url` - The original database URL containing the placeholder
/// * `new_db_name` - The new database name to replace the placeholder
///
/// # Returns
///
/// A new database URL with the updated database name.
fn replace_db_name(db_url: &str, new_db_name: &str) -> String {
    db_url.replace("test_template_db", new_db_name)
}

/// Creates a connection pool for a given database URL
///
/// This function connects to the database specified by `db_url`
/// and returns a connection pool.
///
/// # Arguments
///
/// * `db_url` - The database URL to connect to
///
/// # Returns
///
/// A connection pool (`PgPool`) for the database.
async fn connect_to_db(db_url: &str) -> PgPool {
    PgPool::connect(db_url)
        .await
        .unwrap_or_else(|_| panic!("Failed to connect to database: {db_url}"))
}

/// Creates a new database with the given name
///
/// This function executes a `CREATE DATABASE` command using the provided
/// admin connection pool.
///
/// # Arguments
///
/// * `admin_pool` - The connection pool for the admin database
/// * `db_name` - The name of the new database to create
async fn create_database(admin_pool: &PgPool, db_name: &str) {
    admin_pool
        .execute(format!(r#"CREATE DATABASE "{}""#, db_name).as_str())
        .await
        .expect("Failed to create test database");
}

/// Drops an existing database with the given name
///
/// This function executes a `DROP DATABASE` command using the provided
/// admin connection pool.
///
/// # Arguments
///
/// * `admin_pool` - The connection pool for the admin database
/// * `db_name` - The name of the database to drop
async fn drop_database(admin_pool: &PgPool, db_name: &str) {
    admin_pool
        .execute(format!(r#"DROP DATABASE "{}" WITH (FORCE)"#, db_name).as_str())
        .await
        .expect("Failed to drop test database");
}

/// Sets up a test database and returns the connection pool and database name
///
/// This function creates a new test database, runs migrations, and
/// returns the connection pool and the name of the database.
///
/// # Returns
///
/// A tuple containing:
/// * `PgPool` - The connection pool for the test database
/// * `String` - The name of the test database
async fn setup_test_db(test_id: Uuid) -> (PgPool, String) {
    load_env();

    let test_template_url =
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for tests");
    let admin_db_url = replace_db_name(&test_template_url, "postgres");

    let test_db_name = format!("test_db_{}", test_id);
    let test_db_url = replace_db_name(&test_template_url, &test_db_name);

    let admin_pool = connect_to_db(&admin_db_url).await;
    create_database(&admin_pool, &test_db_name).await;

    let test_pool = connect_to_db(&test_db_url).await;

    sqlx::migrate!("./migrations")
        .run(&test_pool)
        .await
        .expect("Failed to run migrations");

    (test_pool, test_db_name)
}

async fn load_template_data(test_pool: &PgPool) {
    sqlx::query_file!("tests/test_data/sql/test_data.sql")
        .execute(test_pool)
        .await
        .expect("Failed to run test migrations");
}

/// Sets env variables for directories used for storing the files and
/// creates these directories using facades
async fn create_test_resources_dir(test_id: Uuid) -> String {
    let current_test_path = format!("./tests_resources/test-{test_id}");
    env::set_var(
        "VIDEO_DIRECTORY_PATH",
        format!("{current_test_path}/videos"),
    );
    env::set_var(
        "THUMBNAIL_DIRECTORY_PATH",
        format!("{current_test_path}/thumbnails"),
    );
    env::set_var("TEMP_DIRECTORY_PATH", format!("{current_test_path}/temp"));

    VideoFacade::create_dirs()
        .await
        .expect("Failed to create test resource folders");
    TempFileFacade::create_temp_directory()
        .await
        .expect("Failed to create temp folder");

    current_test_path
}

/// Deletes all files in test_resources folder
async fn delete_test_resources_dir(test_folders_root: String) {
    std::fs::remove_dir_all(test_folders_root).expect("Failed to remove test resources directory");
}

/// Tears down the test database
///
/// This function closes the connection pool and drops the test database.
///
/// # Arguments
///
/// * `test_pool` - The connection pool for the test database
/// * `test_db_name` - The name of the test database to drop
async fn teardown_test_db(test_pool: PgPool, test_db_name: &str) {
    test_pool.close().await;

    let test_template_url =
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for tests");
    let admin_db_url = replace_db_name(&test_template_url, "postgres");

    let admin_pool = connect_to_db(&admin_db_url).await;
    drop_database(&admin_pool, test_db_name).await;
}
