use crate::persistence::entities::user::{User, UserName};
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait UserRepo: Debug {
    async fn list_users(&self) -> anyhow::Result<Vec<User>>;
    async fn fetch_usernames_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<UserName>>;
}

#[derive(Debug, Clone)]
pub struct PostgresUserRepo {
    pg_pool: PgPool,
}

impl PostgresUserRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl UserRepo for PostgresUserRepo {
    async fn list_users(&self) -> anyhow::Result<Vec<User>> {
        let users = sqlx::query_as!(User, "SELECT * FROM user_table")
            .fetch_all(&self.pg_pool)
            .await?;

        Ok(users)
    }

    async fn fetch_usernames_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<UserName>> {
        let users = sqlx::query_as!(
            UserName,
            r#"
            SELECT id, username
            FROM user_table
            WHERE id IN (SELECT unnest($1::integer[]))
            "#,
            &ids
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(users)
    }
}
