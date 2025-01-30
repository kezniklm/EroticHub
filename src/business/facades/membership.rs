use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::types::BigDecimal;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use crate::business::mappers::generic::ToMappedList;
use crate::business::models::deal::DealModel;
use crate::business::models::error::{AppError, MapToAppError};
use crate::business::models::membership_details::MembershipDetails;
use crate::business::models::paying_member::PayingMemberModel;
use crate::business::models::payment_method::PaymentMethodModel;
use crate::business::Result;
use crate::persistence::repositories::deal::{DealRepo, UpdateDealInput};
use crate::persistence::repositories::paying_member::PayingMemberRepo;
use crate::persistence::repositories::payment_method::{NewPaymentMethod, PaymentMethodRepo};
use crate::persistence::repositories::unit_of_work::UnitOfWork;

#[derive(Deserialize)]
pub struct PaymentMethodInput {
    pub card_number: String,
    pub card_expiration_date: String,
    pub card_cvc: String,
    pub back_to: String,
}

#[derive(Deserialize)]
pub struct DealInput {
    pub label: String,
    pub number_of_months: String,
    pub price_per_month: String,
}

#[async_trait]
pub trait MembershipFacadeTrait {
    async fn has_payment_method(&self, user_id: i32) -> Result<bool>;
    async fn get_payment_method(&self, user_id: i32) -> Result<Option<PaymentMethodModel>>;
    async fn get_membership_details(&self, user_id: i32) -> Result<MembershipDetails>;
    async fn change_payment_method(&self, user_id: i32, input: PaymentMethodInput) -> Result<i32>;
    async fn get_deals(&self) -> Result<Vec<DealModel>>;
    async fn get_deal(&self, deal_id: i32) -> Result<Option<DealModel>>;
    async fn pay(&self, user_id: i32, deal_id: i32) -> Result<()>;
    async fn edit_deal(&self, deal_id: i32, input: DealInput) -> Result<()>;
    async fn delete_deal(&self, deal_id: i32) -> Result<()>;
    async fn add_deal(&self, input: DealInput) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct MembershipFacade {
    unit_of_work: Arc<dyn UnitOfWork + Sync + Send>,
    paying_member_repository: Arc<dyn PayingMemberRepo + Send + Sync>,
    payment_method_repository: Arc<dyn PaymentMethodRepo + Send + Sync>,
    deal_repository: Arc<dyn DealRepo + Send + Sync>,
}

impl MembershipFacade {
    pub fn new(
        unit_of_work: Arc<dyn UnitOfWork + Sync + Send>,
        paying_member_repository: Arc<dyn PayingMemberRepo + Send + Sync>,
        payment_method_repository: Arc<dyn PaymentMethodRepo + Send + Sync>,
        deal_repository: Arc<dyn DealRepo + Send + Sync>,
    ) -> Self {
        Self {
            unit_of_work,
            paying_member_repository,
            payment_method_repository,
            deal_repository,
        }
    }
}

#[async_trait]
impl MembershipFacadeTrait for MembershipFacade {
    async fn has_payment_method(&self, user_id: i32) -> Result<bool> {
        let payment_method = self.get_payment_method(user_id).await?;
        Ok(payment_method.is_some())
    }

    async fn get_payment_method(&self, user_id: i32) -> Result<Option<PaymentMethodModel>> {
        let mut tx = self.unit_of_work.begin().await?;

        let paying_member = self
            .paying_member_repository
            .get_paying_member(user_id, Some(&mut tx))
            .await?;

        let payment_method = match paying_member {
            Some(paying_member) => {
                self.payment_method_repository
                    .get_payment_method(paying_member.id, Some(&mut tx))
                    .await?
            }
            None => None,
        };

        self.unit_of_work.commit(tx).await?;

        Ok(payment_method.map(PaymentMethodModel::from))
    }

    async fn get_membership_details(&self, user_id: i32) -> Result<MembershipDetails> {
        let mut tx = self.unit_of_work.begin().await?;

        let paying_member = self
            .paying_member_repository
            .get_paying_member(user_id, Some(&mut tx))
            .await?;

        let payment_method = match &paying_member {
            Some(paying_member) => {
                self.payment_method_repository
                    .get_payment_method(paying_member.id, Some(&mut tx))
                    .await?
            }
            None => None,
        };

        self.unit_of_work.commit(tx).await?;

        Ok(MembershipDetails {
            paying_member: paying_member.map(PayingMemberModel::from),
            payment_method: payment_method.map(PaymentMethodModel::from),
        })
    }

    async fn change_payment_method(&self, user_id: i32, input: PaymentMethodInput) -> Result<i32> {
        let mut tx = self.unit_of_work.begin().await?;

        let paying_member_id = match self
            .paying_member_repository
            .get_paying_member(user_id, Some(&mut tx))
            .await?
        {
            Some(paying_member) => paying_member.id,
            None => {
                self.paying_member_repository
                    .add_paying_member(user_id, Some(&mut tx))
                    .await?
            }
        };

        let card_expiration_date =
            NaiveDate::parse_from_str(&format!("01/{}", input.card_expiration_date), "%d/%m/%Y")
                .app_error("Invalid card expiration date")?;
        let payment_method_id = self
            .payment_method_repository
            .change_payment_method(
                NewPaymentMethod {
                    paying_member_id,
                    card_number: input.card_number,
                    card_expiration_date,
                    card_cvc: input.card_cvc,
                },
                Some(&mut tx),
            )
            .await?;

        self.unit_of_work.commit(tx).await?;

        Ok(payment_method_id)
    }

    async fn get_deals(&self) -> Result<Vec<DealModel>> {
        let deals = self.deal_repository.get_deals(None).await?;

        let deal_models = deals.to_mapped_list(DealModel::from);
        Ok(deal_models)
    }

    async fn get_deal(&self, deal_id: i32) -> Result<Option<DealModel>> {
        let deal = self.deal_repository.get_deal(deal_id, None).await?;

        Ok(deal.map(DealModel::from))
    }

    async fn pay(&self, user_id: i32, deal_id: i32) -> Result<()> {
        let mut tx = self.unit_of_work.begin().await?;

        let deal = self
            .deal_repository
            .get_deal(deal_id, Some(&mut tx))
            .await?
            .ok_or_else(|| {
                AppError::new(
                    "Deal not found",
                    crate::business::models::error::AppErrorKind::NotFound,
                )
            })?;

        self.paying_member_repository
            .extend_validity(user_id, deal.number_of_months, Some(&mut tx))
            .await?;

        self.unit_of_work.commit(tx).await?;

        Ok(())
    }

    async fn edit_deal(&self, deal_id: i32, input: DealInput) -> Result<()> {
        let label = input.label.clone();
        let number_of_months = input
            .number_of_months
            .parse()
            .app_error("Invalid number of months")?;
        let price_per_month =
            BigDecimal::from_str(&input.price_per_month).app_error("Invalid price per month")?;

        self.deal_repository
            .update_deal(
                deal_id,
                UpdateDealInput {
                    label,
                    number_of_months,
                    price_per_month,
                },
                None,
            )
            .await?;

        Ok(())
    }

    async fn delete_deal(&self, deal_id: i32) -> Result<()> {
        self.deal_repository.delete_deal(deal_id, None).await?;
        Ok(())
    }

    async fn add_deal(&self, input: DealInput) -> Result<()> {
        let label = input.label.clone();
        let number_of_months = input
            .number_of_months
            .parse()
            .app_error("Invalid number of months")?;
        let price_per_month =
            BigDecimal::from_str(&input.price_per_month).app_error("Invalid price per month")?;

        self.deal_repository
            .add_deal(
                UpdateDealInput {
                    label,
                    number_of_months,
                    price_per_month,
                },
                None,
            )
            .await?;

        Ok(())
    }
}
