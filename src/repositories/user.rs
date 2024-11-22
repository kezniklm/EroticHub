use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::user::User;

#[async_trait]
pub trait UserRepo {
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
