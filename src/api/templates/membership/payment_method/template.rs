use askama_actix::Template;

#[derive(Template)]
#[template(path = "membership/payment_method/index.html")]
pub struct PaymentMethodTemplate {
    pub user_id: i32,
    pub has_payment_method: bool,
}
