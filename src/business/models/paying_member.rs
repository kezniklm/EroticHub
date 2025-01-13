use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PayingMemberModel {
    pub id: i32,
    pub user_id: i32,
    pub valid_until: Option<String>,
    pub is_valid: bool,
    pub payment_method_id: Option<i32>,
}
