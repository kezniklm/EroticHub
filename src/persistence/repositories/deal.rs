use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

use crate::persistence::entities::deal::{Deal, DealExtended};

#[async_trait]
pub trait DealRepo: Debug {
    async fn get_deals(&self) -> anyhow::Result<Vec<DealExtended>>;
    async fn get_deal(&self, deal_id: i32) -> anyhow::Result<Option<DealExtended>>;
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
    async fn get_deals(&self) -> anyhow::Result<Vec<DealExtended>> {
        let deals = sqlx::query_as!(Deal, "SELECT * FROM deal ORDER BY number_of_months ASC")
            .fetch_all(&self.pg_pool)
            .await?;

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

    async fn get_deal(&self, deal_id: i32) -> anyhow::Result<Option<DealExtended>> {
        // not done using a query to get DealExtended instead of Deal
        let deals = self.get_deals().await?;
        Ok(deals.into_iter().find(|deal| deal.id == deal_id))
    }
}
