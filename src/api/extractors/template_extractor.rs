use actix_http::Payload;
use actix_web::{Error, FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct TemplateReq {
    pub return_template: bool,
}

impl FromRequest for TemplateReq {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let query_str = req.query_string();
        if query_str.contains("get_template=false") {
            return Box::pin(async {
                Ok(Self {
                    return_template: false,
                })
            });
        }

        Box::pin(async {
            Ok(Self {
                return_template: true,
            })
        })
    }
}
