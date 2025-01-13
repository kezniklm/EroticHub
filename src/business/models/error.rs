use crate::persistence::entities::error::DatabaseError;
use actix_identity::error::{GetIdentityError, LoginError};
use actix_session::SessionInsertError;
use std::fmt::{Display, Formatter};
use validator::ValidationError;

#[derive(Debug)]
pub(crate) struct AppError {
    pub message: String,
    pub error: AppErrorKind,
}

impl AppError {
    pub fn new(message: &str, error: AppErrorKind) -> Self {
        Self {
            message: message.to_string(),
            error,
        }
    }
}

#[derive(Debug)]
pub enum AppErrorKind {
    WrongMimeType,
    InternalServerError,
    BadRequestError,
    Unauthorized,
}

pub trait MapToAppError<T> {
    /// If result contains Error, then it is mapped to `AppError` with given message. \
    /// Ok result is not touched.
    fn app_error(self, message: &str) -> Result<T, AppError>;
}

impl<T, E> MapToAppError<T> for Result<T, E> {
    fn app_error(self, message: &str) -> Result<T, AppError> {
        match self {
            Ok(obj) => Ok(obj),
            Err(_) => Err(AppError::new(message, AppErrorKind::InternalServerError)),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.error {
            AppErrorKind::WrongMimeType => write!(f, "Unsupported MimeType: {}", self.message),
            AppErrorKind::BadRequestError => write!(f, "Bad request: {}", self.message),
            AppErrorKind::Unauthorized => write!(f, "Unauthorized: {}", self.message),
            AppErrorKind::InternalServerError => {
                write!(f, "Internal server error: {}", self.message)
            }
        }
    }
}

impl From<DatabaseError> for AppError {
    fn from(value: DatabaseError) -> Self {
        Self::new(&value.error, AppErrorKind::InternalServerError)
    }
}

impl From<LoginError> for AppError {
    fn from(_: LoginError) -> Self {
        AppError::new("Login failed.", AppErrorKind::Unauthorized)
    }
}

impl From<ValidationError> for AppError {
    fn from(_: ValidationError) -> Self {
        AppError::new("Validation failed", AppErrorKind::BadRequestError)
    }
}

impl From<SessionInsertError> for AppError {
    fn from(value: SessionInsertError) -> Self {
        Self::new(&value.to_string(), AppErrorKind::InternalServerError)
    }
}

impl From<GetIdentityError> for AppError {
    fn from(value: GetIdentityError) -> Self {
        Self::new(&value.to_string(), AppErrorKind::Unauthorized)
    }
}
