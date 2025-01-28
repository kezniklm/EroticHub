use async_trait::async_trait;
use sqlx::types::BigDecimal;
use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Debug;

use crate::persistence::entities::deal::{Deal, DealExtended};
use crate::persistence::Result;

pub struct UpdateDealInput {
    pub label: String,
    pub number_of_months: i32,
    pub price_per_month: BigDecimal,
}

#[async_trait]
pub trait DealRepo: Debug {
    async fn get_deals(&self, tx: Option<&mut Transaction<Postgres>>) -> Result<Vec<DealExtended>>;
    async fn get_deal(
        &self,
        deal_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<DealExtended>>;
    async fn update_deal(
        &self,
        deal_id: i32,
        input: UpdateDealInput,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct PostgresDealRepo {
    pg_pool: PgPool,
}

impl PostgresDealRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl DealRepo for PostgresDealRepo {
    async fn get_deals(&self, tx: Option<&mut Transaction<Postgres>>) -> Result<Vec<DealExtended>> {
        let query = sqlx::query_as!(Deal, "SELECT * FROM deal ORDER BY number_of_months ASC");
        let deals = match tx {
            Some(tx) => query.fetch_all(tx.as_mut()).await,
            None => query.fetch_all(&self.pg_pool).await,
        }?;

        let max_price_per_month = deals
            .iter()
            .map(|deal| deal.price_per_month.clone())
            .max()
            .unwrap();

        let deals_extended = deals
            .into_iter()
            .map(|deal| DealExtended {
                id: deal.id,
                label: deal.label,
                price_per_month: deal.price_per_month.clone(),
                number_of_months: deal.number_of_months,
                total_price: deal.price_per_month * deal.number_of_months,
                total_price_without_discount: max_price_per_month.clone() * deal.number_of_months,
            })
            .collect();

        Ok(deals_extended)
    }

    async fn get_deal(
        &self,
        deal_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<DealExtended>> {
        // not done using a query to get DealExtended instead of Deal
        let deals = self.get_deals(tx).await?;
        Ok(deals.into_iter().find(|deal| deal.id == deal_id))
    }

    async fn update_deal(
        &self,
        deal_id: i32,
        input: UpdateDealInput,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<()> {
        let query = sqlx::query!(
            "UPDATE deal SET label = $1, number_of_months = $2, price_per_month = $3 WHERE id = $4",
            input.label,
            input.number_of_months,
            input.price_per_month,
            deal_id
        );
        match tx {
            Some(tx) => query.execute(tx.as_mut()).await,
            None => query.execute(&self.pg_pool).await,
        }?;

        Ok(())
    }
}
