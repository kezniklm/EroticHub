use crate::business::models::membership_details::MembershipDetails;
use askama_actix::Template;

#[derive(Template)]
#[template(path = "membership/details/index.html")]
pub struct MembershipDetailsTemplate {
    pub membership_details: MembershipDetails,
}
