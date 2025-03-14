use askama_actix::Template;

use crate::business::models::{deal::DealModel, membership_details::MembershipDetails};

#[derive(Template)]
#[template(path = "membership/deal/index.html")]
pub struct DealTemplate {
    pub membership_details: MembershipDetails,
    pub deals: Vec<DealModel>,
}
