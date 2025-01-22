use sqlx::types::BigDecimal;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct DealModel {
    pub id: i32,
    pub label: String,
    pub price_per_month: BigDecimal,
    pub number_of_months: i32,
    pub total_price: BigDecimal,
    pub total_price_without_discount: BigDecimal,
}
