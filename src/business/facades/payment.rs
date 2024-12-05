use crate::persistence::repositories::paying_member::PayingMemberRepo;
use crate::persistence::repositories::payment_method::{NewPaymentMethod, PaymentMethodRepo};
use async_trait::async_trait;
use serde::Deserialize;
use std::fmt::Debug;
use std::sync::Arc;

// TODO: add fields
#[derive(Deserialize)]
pub struct PaymentMethodInput {}

#[async_trait]
pub trait PaymentFacadeTrait {
    async fn change_payment_method(
        &self,
        user_id: i32,
        input: PaymentMethodInput,
    ) -> anyhow::Result<i32>;
}

#[derive(Debug, Clone)]
pub struct PaymentFacade {
    paying_member_repository: Arc<dyn PayingMemberRepo + Send + Sync>,
    payment_method_repository: Arc<dyn PaymentMethodRepo + Send + Sync>,
}

impl PaymentFacade {
    pub fn new(
        paying_member_repository: Arc<dyn PayingMemberRepo + Send + Sync>,
        payment_method_repository: Arc<dyn PaymentMethodRepo + Send + Sync>,
    ) -> Self {
        Self {
            paying_member_repository,
            payment_method_repository,
        }
    }
}

#[async_trait]
impl PaymentFacadeTrait for PaymentFacade {
    async fn change_payment_method(
        &self,
        user_id: i32,
        input: PaymentMethodInput,
    ) -> anyhow::Result<i32> {
        let paying_member_id = match self
            .paying_member_repository
            .get_paying_member(user_id)
            .await?
        {
            Some(paying_member) => paying_member.id,
            None => {
                self.paying_member_repository
                    .add_paying_member(user_id)
                    .await?
            }
        };

        let payment_method_id = self
            .payment_method_repository
            .change_payment_method(NewPaymentMethod { paying_member_id })
            .await?;

        Ok(payment_method_id)
    }
}
