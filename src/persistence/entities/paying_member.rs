use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PayingMember {
    pub id: i32,
    pub user_id: i32,
    pub valid_until: Option<DateTime<Utc>>,
    pub payment_method_id: Option<i32>,
}
