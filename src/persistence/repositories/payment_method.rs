use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Debug;

use crate::persistence::entities::payment_method::PaymentMethod;
use crate::persistence::Result;

pub struct NewPaymentMethod {
    pub paying_member_id: i32,
    pub card_number: String,
    pub card_expiration_date: NaiveDate,
    pub card_cvc: String,
}

#[async_trait]
pub trait PaymentMethodRepo: Debug {
    async fn has_payment_method(
        &self,
        paying_member_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<bool>;
    async fn get_payment_method(
        &self,
        paying_member_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<PaymentMethod>>;
    async fn change_payment_method_tx(
        &self,
        new_payment_method: NewPaymentMethod,
        tx: &mut Transaction<Postgres>,
    ) -> Result<i32>;
    async fn change_payment_method(
        &self,
        new_payment_method: NewPaymentMethod,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<i32>;
}

#[derive(Debug, Clone)]
pub struct PostgresPaymentMethodRepo {
    pg_pool: PgPool,
}

impl PostgresPaymentMethodRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl PaymentMethodRepo for PostgresPaymentMethodRepo {
    async fn has_payment_method(
        &self,
        paying_member_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<bool> {
        let payment_method = self.get_payment_method(paying_member_id, tx).await?;
        Ok(payment_method.is_some())
    }

    async fn get_payment_method(
        &self,
        paying_member_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<PaymentMethod>> {
        let query = sqlx::query_as!(
            PaymentMethod,
            "SELECT * FROM payment_method WHERE paying_member_id = $1",
            paying_member_id
        );
        let payment_method = match tx {
            Some(tx) => query.fetch_optional(tx.as_mut()).await,
            None => query.fetch_optional(&self.pg_pool).await,
        }?;

        Ok(payment_method)
    }

    async fn change_payment_method_tx(
        &self,
        new_payment_method: NewPaymentMethod,
        tx: &mut Transaction<Postgres>,
    ) -> Result<i32> {
        if let Some(existing_payment_method) = sqlx::query!(
            "SELECT id FROM payment_method WHERE paying_member_id = $1",
            new_payment_method.paying_member_id
        )
        .fetch_optional(tx.as_mut())
        .await?
        {
            sqlx::query!(
                "DELETE FROM payment_method WHERE id = $1",
                existing_payment_method.id
            )
            .execute(tx.as_mut())
            .await?;
        }

        let new_payment_method_id = sqlx::query!(
            "INSERT INTO payment_method (paying_member_id, card_number, card_expiration_date, card_cvc)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
            new_payment_method.paying_member_id,
            new_payment_method.card_number,
            new_payment_method.card_expiration_date,
            new_payment_method.card_cvc
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;

        sqlx::query!(
            "UPDATE paying_member SET payment_method_id = $1 WHERE id = $2",
            new_payment_method_id,
            new_payment_method.paying_member_id
        )
        .execute(tx.as_mut())
        .await?;

        Ok(new_payment_method_id)
    }

    async fn change_payment_method(
        &self,
        new_payment_method: NewPaymentMethod,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<i32> {
        match tx {
            Some(tx) => self.change_payment_method_tx(new_payment_method, tx).await,
            None => {
                let mut tx = self.pg_pool.begin().await?;
                let new_payment_method_id = self
                    .change_payment_method_tx(new_payment_method, &mut tx)
                    .await?;
                tx.commit().await?;
                Ok(new_payment_method_id)
            }
        }
    }
}
