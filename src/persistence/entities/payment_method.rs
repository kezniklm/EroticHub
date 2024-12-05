#[derive(sqlx::FromRow)]
pub struct PaymentMethod {
    pub id: i32,
    pub paying_member_id: i32,
}
