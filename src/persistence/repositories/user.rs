use crate::persistence::entities::user::User;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait UserRepo: Debug {
    async fn list_users(&self) -> anyhow::Result<Vec<User>>;
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
}
