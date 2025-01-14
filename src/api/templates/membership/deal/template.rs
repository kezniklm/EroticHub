use askama_actix::Template;

use crate::business::models::membership_details::MembershipDetails;

#[derive(Template)]
#[template(path = "membership/deal/index.html")]
pub struct DealTemplate {
    pub user_id: i32,
    pub membership_details: MembershipDetails,
}
