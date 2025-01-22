use crate::{business::models::deal::DealModel, persistence::entities::deal::DealExtended};

impl From<DealExtended> for DealModel {
    fn from(deal: DealExtended) -> Self {
        DealModel {
            id: deal.id,
            label: deal.label,
            price_per_month: deal.price_per_month,
            number_of_months: deal.number_of_months,
            total_price: deal.total_price,
            total_price_without_discount: deal.total_price_without_discount,
        }
    }
}
