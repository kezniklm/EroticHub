use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use std::fmt::Debug;

use crate::persistence::entities::payment_method::PaymentMethod;

pub struct NewPaymentMethod {
    pub paying_member_id: i32,
    pub card_number: String,
    pub card_expiration_date: NaiveDate,
    pub card_cvc: String,
}

#[async_trait]
pub trait PaymentMethodRepo: Debug {
    async fn has_payment_method(&self, paying_member_id: i32) -> anyhow::Result<bool>;
    async fn get_payment_method(
        &self,
        paying_member_id: i32,
    ) -> anyhow::Result<Option<PaymentMethod>>;
    async fn change_payment_method(
        &self,
        new_payment_method: NewPaymentMethod,
    ) -> anyhow::Result<i32>;
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
    async fn has_payment_method(&self, paying_member_id: i32) -> anyhow::Result<bool> {
        let payment_method = sqlx::query!(
            "SELECT id FROM payment_method WHERE paying_member_id = $1",
            paying_member_id
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(payment_method.is_some())
    }

    async fn get_payment_method(
        &self,
        paying_member_id: i32,
    ) -> anyhow::Result<Option<PaymentMethod>> {
        let payment_method = sqlx::query_as!(
            PaymentMethod,
            "SELECT * FROM payment_method WHERE paying_member_id = $1",
            paying_member_id
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(payment_method)
    }

    async fn change_payment_method(
        &self,
        new_payment_method: NewPaymentMethod,
    ) -> anyhow::Result<i32> {
        let mut tx = self.pg_pool.begin().await?;

        if let Some(existing_payment_method) = sqlx::query!(
            "SELECT id FROM payment_method WHERE paying_member_id = $1",
            new_payment_method.paying_member_id
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            sqlx::query!(
                "DELETE FROM payment_method WHERE id = $1",
                existing_payment_method.id
            )
            .execute(&mut *tx)
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
        .fetch_one(&mut *tx)
        .await?
        .id;

        sqlx::query!(
            "UPDATE paying_member SET payment_method_id = $1 WHERE id = $2",
            new_payment_method_id,
            new_payment_method.paying_member_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(new_payment_method_id)
    }
}
