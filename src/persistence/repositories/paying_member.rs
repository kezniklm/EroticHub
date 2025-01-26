use async_trait::async_trait;
use sqlx::{query, PgPool, Postgres, Transaction};
use std::fmt::Debug;

use crate::persistence::entities::paying_member::PayingMember;
use crate::persistence::Result;

#[async_trait]
pub trait PayingMemberRepo: Debug {
    async fn get_paying_member(
        &self,
        user_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<PayingMember>>;
    async fn add_paying_member_tx(
        &self,
        user_id: i32,
        tx: &mut Transaction<Postgres>,
    ) -> Result<i32>;
    async fn add_paying_member(
        &self,
        user_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<i32>;
    async fn extend_validity(
        &self,
        user_id: i32,
        number_of_months: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct PostgresPayingMemberRepo {
    pg_pool: PgPool,
}

impl PostgresPayingMemberRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl PayingMemberRepo for PostgresPayingMemberRepo {
    async fn get_paying_member(
        &self,
        user_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<PayingMember>> {
        let query = sqlx::query_as!(
            PayingMember,
            "SELECT * FROM paying_member WHERE user_id = $1",
            user_id
        );
        let paying_member = match tx {
            Some(tx) => query.fetch_optional(tx.as_mut()).await,
            None => query.fetch_optional(&self.pg_pool).await,
        }?;

        Ok(paying_member)
    }

    async fn add_paying_member_tx(
        &self,
        user_id: i32,
        tx: &mut Transaction<Postgres>,
    ) -> Result<i32> {
        let paying_member_id = query!(
            "INSERT INTO paying_member (user_id) VALUES ($1) RETURNING id",
            user_id
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;

        query!(
            "UPDATE user_table SET paying_member_id = $1 WHERE id = $2",
            paying_member_id,
            user_id
        )
        .execute(tx.as_mut())
        .await?;

        Ok(paying_member_id)
    }

    async fn add_paying_member(
        &self,
        user_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<i32> {
        match tx {
            Some(tx) => self.add_paying_member_tx(user_id, tx).await,
            None => {
                let mut tx = self.pg_pool.begin().await?;
                let paying_member_id = self.add_paying_member_tx(user_id, &mut tx).await?;
                tx.commit().await?;
                Ok(paying_member_id)
            }
        }
    }

    async fn extend_validity(
        &self,
        user_id: i32,
        number_of_months: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()> {
        let query = sqlx::query!(
            "UPDATE paying_member 
             SET valid_until = COALESCE(valid_until, NOW()) + interval '1 month' * $1 
             WHERE user_id = $2",
            number_of_months as f64,
            user_id,
        );
        match tx {
            Some(tx) => query.execute(tx.as_mut()).await,
            None => query.execute(&self.pg_pool).await,
        }?;

        Ok(())
    }
}
