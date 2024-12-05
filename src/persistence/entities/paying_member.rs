use chrono::{DateTime, Local};

#[derive(sqlx::FromRow)]
pub struct PayingMember {
    pub id: i32,
    pub user_id: i32,
    pub valid_until: DateTime<Local>,
    pub payment_method_id: Option<i32>,
}
