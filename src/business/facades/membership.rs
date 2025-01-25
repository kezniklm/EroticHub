use crate::business::mappers::generic::ToMappedList;
use crate::business::models::deal::DealModel;
use crate::business::models::membership_details::MembershipDetails;
use crate::business::models::paying_member::PayingMemberModel;
use crate::business::models::payment_method::PaymentMethodModel;
use crate::persistence::repositories::deal::DealRepo;
use crate::persistence::repositories::paying_member::PayingMemberRepo;
use crate::persistence::repositories::payment_method::{NewPaymentMethod, PaymentMethodRepo};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Deserialize;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PaymentMethodInput {
    pub card_number: String,
    pub card_expiration_date: String,
    pub card_cvc: String,
    pub back_to: String,
}

#[async_trait]
pub trait MembershipFacadeTrait {
    async fn has_payment_method(&self, user_id: i32) -> anyhow::Result<bool>;
    async fn get_payment_method(&self, user_id: i32) -> anyhow::Result<Option<PaymentMethodModel>>;
    async fn get_membership_details(&self, user_id: i32) -> anyhow::Result<MembershipDetails>;
    async fn change_payment_method(
        &self,
        user_id: i32,
        input: PaymentMethodInput,
    ) -> anyhow::Result<i32>;
    async fn get_deals(&self) -> anyhow::Result<Vec<DealModel>>;
    async fn get_deal(&self, deal_id: i32) -> anyhow::Result<Option<DealModel>>;
    async fn pay(&self, user_id: i32, deal_id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct MembershipFacade {
    paying_member_repository: Arc<dyn PayingMemberRepo + Send + Sync>,
    payment_method_repository: Arc<dyn PaymentMethodRepo + Send + Sync>,
    deal_repository: Arc<dyn DealRepo + Send + Sync>,
}

impl MembershipFacade {
    pub fn new(
        paying_member_repository: Arc<dyn PayingMemberRepo + Send + Sync>,
        payment_method_repository: Arc<dyn PaymentMethodRepo + Send + Sync>,
        deal_repository: Arc<dyn DealRepo + Send + Sync>,
    ) -> Self {
        Self {
            paying_member_repository,
            payment_method_repository,
            deal_repository,
        }
    }
}

// TODO: use payment_method_id (paying_member) and paying_member_id (user) fields accordingly
#[async_trait]
impl MembershipFacadeTrait for MembershipFacade {
    async fn has_payment_method(&self, user_id: i32) -> anyhow::Result<bool> {
        // TODO: refactor to a transaction
        let paying_member = self
            .paying_member_repository
            .get_paying_member(user_id)
            .await?;

        return match paying_member {
            Some(paying_member) => {
                self.payment_method_repository
                    .has_payment_method(paying_member.id)
                    .await
            }
            None => Ok(false),
        };
    }

    async fn get_payment_method(&self, user_id: i32) -> anyhow::Result<Option<PaymentMethodModel>> {
        // TODO: refactor to a transaction
        let paying_member = self
            .paying_member_repository
            .get_paying_member(user_id)
            .await?;

        return match paying_member {
            Some(paying_member) => self
                .payment_method_repository
                .get_payment_method(paying_member.id)
                .await
                .map(|option_pm| option_pm.map(PaymentMethodModel::from)),
            None => Ok(None),
        };
    }

    async fn get_membership_details(&self, user_id: i32) -> anyhow::Result<MembershipDetails> {
        // TODO: refactor to a transaction
        let paying_member = self
            .paying_member_repository
            .get_paying_member(user_id)
            .await?;

        let payment_method = match &paying_member {
            Some(paying_member) => {
                self.payment_method_repository
                    .get_payment_method(paying_member.id)
                    .await?
            }
            None => None,
        };

        Ok(MembershipDetails {
            paying_member: paying_member.map(PayingMemberModel::from),
            payment_method: payment_method.map(PaymentMethodModel::from),
        })
    }

    async fn change_payment_method(
        &self,
        user_id: i32,
        input: PaymentMethodInput,
    ) -> anyhow::Result<i32> {
        // TODO: refactor to a transaction
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

        let card_expiration_date =
            NaiveDate::parse_from_str(&format!("01/{}", input.card_expiration_date), "%d/%m/%Y")?;
        let payment_method_id = self
            .payment_method_repository
            .change_payment_method(NewPaymentMethod {
                paying_member_id,
                card_number: input.card_number,
                card_expiration_date,
                card_cvc: input.card_cvc,
            })
            .await?;

        Ok(payment_method_id)
    }

    async fn get_deals(&self) -> anyhow::Result<Vec<DealModel>> {
        // TODO: refactor to a transaction
        let deals = self.deal_repository.get_deals().await?;
        let deal_models = deals.to_mapped_list(DealModel::from);
        Ok(deal_models)
    }

    async fn get_deal(&self, deal_id: i32) -> anyhow::Result<Option<DealModel>> {
        let deal = self.deal_repository.get_deal(deal_id).await?;
        Ok(deal.map(DealModel::from))
    }

    async fn pay(&self, user_id: i32, deal_id: i32) -> anyhow::Result<()> {
        // TODO: refactor to a transaction

        let deal = match self.deal_repository.get_deal(deal_id).await? {
            Some(deal) => deal,
            None => return Err(anyhow::anyhow!("Deal not found")),
        };

        self.paying_member_repository
            .extend_validity(user_id, deal.number_of_months)
            .await?;

        Ok(())
    }
}
