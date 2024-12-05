use crate::api::templates::user::list::template::UserListTemplate;
use crate::business::facades::payment::{PaymentFacade, PaymentFacadeTrait, PaymentMethodInput};
use actix_web::{web, HttpResponse, Responder};
use askama::Template;

pub async fn change_payment_method(
    payment_facade: web::Data<PaymentFacade>,
    user_id: web::Path<i32>,
    payment_method_input: web::Form<PaymentMethodInput>,
) -> impl Responder {
    match payment_facade
        .change_payment_method(*user_id, payment_method_input.into_inner())
        .await
    {
        Ok(payment_method_id) => HttpResponse::Ok().body(payment_method_id.to_string()),
        Err(err) => {
            // TODO: proper error management
            log::error!("Error while adding payment method: {:#?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
