use askama_actix::Template;

use crate::business::models::{deal::DealModel, membership_details::MembershipDetails};

#[derive(Template)]
#[template(path = "membership/payment/index.html")]
pub struct PaymentTemplate {
    pub membership_details: MembershipDetails,
    pub deal: DealModel,
}
