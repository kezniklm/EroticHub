use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

pub struct NewPaymentMethod {
    pub paying_member_id: i32,
}

#[async_trait]
pub trait PaymentMethodRepo: Debug {
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
            "INSERT INTO payment_method (paying_member_id) VALUES ($1) RETURNING id",
            new_payment_method.paying_member_id
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
