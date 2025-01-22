use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Debug;

#[async_trait]
pub trait UnitOfWork: Debug {
    async fn begin(&self) -> Result<Transaction<'_, Postgres>>;
    async fn commit<'a>(&self, transaction: Transaction<'a, Postgres>) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct PostgresUnitOfWork {
    pg_pool: PgPool,
}

impl PostgresUnitOfWork {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl UnitOfWork for PostgresUnitOfWork {
    async fn begin(&self) -> Result<Transaction<'_, Postgres>> {
        Ok(self.pg_pool.begin().await?)
    }

    async fn commit<'a>(&self, transaction: Transaction<'a, Postgres>) -> Result<()> {
        Ok(transaction.commit().await?)
    }
}
