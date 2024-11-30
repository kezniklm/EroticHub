use log::warn;
use sqlx::{Executor, PgPool};
use std::env;
use test_context::AsyncTestContext;
use uuid::Uuid;

/// A context for managing an asynchronous test database lifecycle
///
/// This struct is used with the `test_context` crate to manage a PostgreSQL
/// test database during tests.
/// To use the AsyncContext use #[test_context(AsyncContext)] along with #[tokio::test]
pub struct AsyncContext {
    pub pg_pool: PgPool,
    pub test_db_name: String,
}

impl AsyncTestContext for AsyncContext {
    /// Sets up the test database and returns the context
    ///
    /// This method is automatically called before tests that use this context.
    /// It creates a new test database and initializes a connection pool.
    async fn setup() -> AsyncContext {
        let (pg_pool, test_db_name) = setup_test_db().await;
        AsyncContext {
            pg_pool,
            test_db_name,
        }
    }

    /// Tears down the test database
    ///
    /// This method is automatically called after tests that use this context.
    /// It drops the test database and cleans up resources.
    async fn teardown(self) {
        teardown_test_db(self.pg_pool, &self.test_db_name).await;
    }
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
        .expect(&format!("Failed to connect to database: {db_url}"))
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
        .execute(format!(r#"DROP DATABASE "{}""#, db_name).as_str())
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
async fn setup_test_db() -> (PgPool, String) {
    load_env();

    let test_template_url =
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for tests");
    let admin_db_url = replace_db_name(&test_template_url, "postgres");

    let test_db_name = format!("test_db_{}", Uuid::new_v4());
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
