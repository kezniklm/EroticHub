use crate::{
    business::models::payment_method::PaymentMethodModel,
    persistence::entities::payment_method::PaymentMethod,
};

impl From<PaymentMethod> for PaymentMethodModel {
    fn from(payment_method: PaymentMethod) -> Self {
        PaymentMethodModel {
            card_number_classified: format!(
                "{}{}",
                "*".repeat(payment_method.card_number.len() - 4),
                &payment_method.card_number[payment_method.card_number.len() - 4..]
            ),
        }
    }
}
