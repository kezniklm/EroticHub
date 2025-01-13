use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentMethod {
    pub id: i32,
    pub paying_member_id: i32,
    pub card_number: String,
    pub card_expiration_date: NaiveDate,
    pub card_cvc: String,
}
