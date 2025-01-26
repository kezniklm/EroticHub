use actix_web::dev::Payload;
use actix_web::http::header::HeaderValue;
use actix_web::{Error, FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Copy, Clone)]
pub struct HtmxRequest {
    pub hx_request: bool,
}

impl FromRequest for HtmxRequest {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let header = req.headers().get("Hx-Request");
        if let Some(header) = header {
            let reference_val = HeaderValue::from_str("true").unwrap();
            if header == reference_val {
                return Box::pin(async { Ok(Self { hx_request: true }) });
            }
        }
        Box::pin(async { Ok(Self { hx_request: false }) })
    }
}
