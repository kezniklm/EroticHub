use crate::{
    business::models::paying_member::PayingMemberModel,
    persistence::entities::paying_member::PayingMember,
};

impl From<PayingMember> for PayingMemberModel {
    fn from(paying_member: PayingMember) -> Self {
        PayingMemberModel {
            id: paying_member.id,
            user_id: paying_member.user_id,
            valid_until: paying_member
                .valid_until
                .map(|valid_until| valid_until.to_rfc2822()),
            is_valid: paying_member
                .valid_until
                .map(|valid_until| valid_until > chrono::Utc::now())
                .unwrap_or(false),
            payment_method_id: paying_member.payment_method_id,
        }
    }
}
