use askama_actix::Template;

#[derive(Template)]
#[template(path = "membership/payment_method/index.html")]
pub struct PaymentMethodTemplate {
    pub has_payment_method: bool,
    pub back_to: String,
}
