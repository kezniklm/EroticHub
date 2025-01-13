use crate::persistence::entities::paying_member::PayingMember;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait PayingMemberRepo: Debug {
    async fn get_paying_member(&self, user_id: i32) -> anyhow::Result<Option<PayingMember>>;
    async fn add_paying_member(&self, user_id: i32) -> anyhow::Result<i32>;
}

#[derive(Debug, Clone)]
pub struct PostgresPayingMemberRepo {
    pg_pool: PgPool,
}

impl PostgresPayingMemberRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl PayingMemberRepo for PostgresPayingMemberRepo {
    async fn get_paying_member(&self, user_id: i32) -> anyhow::Result<Option<PayingMember>> {
        let paying_member = sqlx::query_as!(
            PayingMember,
            "SELECT * FROM paying_member WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pg_pool)
        .await?;
        Ok(paying_member)
    }

    async fn add_paying_member(&self, user_id: i32) -> anyhow::Result<i32> {
        let paying_member_id = sqlx::query!(
            "INSERT INTO paying_member (user_id) VALUES ($1) RETURNING id",
            user_id,
        )
        .fetch_one(&self.pg_pool)
        .await?
        .id;
        println!("add_paying_member: {:?}", paying_member_id);

        Ok(paying_member_id)
    }
}
