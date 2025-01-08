use crate::business::models::error::{AppError, AppErrorKind};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use log::error;

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.error {
            AppErrorKind::WrongMimeType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            AppErrorKind::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorKind::NotFound => StatusCode::NOT_FOUND,
            AppErrorKind::AccessDenied => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        error!("{:#?}", self);
        HttpResponse::build(self.status_code()).body(self.message.clone())
    }
}
