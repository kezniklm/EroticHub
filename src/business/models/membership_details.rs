use serde::{Deserialize, Serialize};

use super::{paying_member::PayingMemberModel, payment_method::PaymentMethodModel};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MembershipDetails {
    pub paying_member: Option<PayingMemberModel>,
    pub payment_method: Option<PaymentMethodModel>,
}
