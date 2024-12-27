use erotic_hub::common::tests::setup::{AsyncContext, EmptyAsyncContext};
use erotic_hub::persistence::entities::user::User;
use sqlx::PgPool;
use test_context::test_context;

mod api;
mod common;

#[test_context(EmptyAsyncContext)]
#[tokio::test]
async fn empty_context_test(ctx: &mut EmptyAsyncContext) -> anyhow::Result<()> {
    let users = query_users(&ctx.pg_pool).await?;
    assert!(
        users.is_empty(),
        "Database in EmptyAsyncContext should be empty"
    );
    Ok(())
}

#[test_context(AsyncContext)]
#[tokio::test]
async fn non_empty_context_test(ctx: &mut AsyncContext) -> anyhow::Result<()> {
    let users = query_users(&ctx.pg_pool).await?;
    assert!(
        !users.is_empty(),
        "Database in AsyncContext shouldn't be empty"
    );
    Ok(())
}

async fn query_users(pg_pool: &PgPool) -> anyhow::Result<Vec<User>> {
    let users = sqlx::query_as!(User, "SELECT * FROM user_table")
        .fetch_all(pg_pool)
        .await?;
    Ok(users)
}
