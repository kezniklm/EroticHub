use crate::persistence::entities::error::DatabaseError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct AppError {
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
    NotFound,
}

pub trait MapToAppError<T> {
    /// If result contains Error, then it is mapped to `AppError` with given message. \
    /// Ok result is not touched.
    /// All errors are mapped to `InternalServerError`
    fn app_error(self, message: &str) -> Result<T, AppError>;

    /// If result contains Error, then it is mapped to `AppError` with given message. \
    /// Ok result is not touched.
    fn app_error_kind(self, message: &str, error: AppErrorKind) -> Result<T, AppError>;
}

impl<T, E> MapToAppError<T> for Result<T, E> {
    fn app_error(self, message: &str) -> Result<T, AppError> {
        match self {
            Ok(obj) => Ok(obj),
            Err(_) => Err(AppError::new(message, AppErrorKind::InternalServerError)),
        }
    }

    fn app_error_kind(self, message: &str, error: AppErrorKind) -> Result<T, AppError> {
        match self {
            Ok(obj) => Ok(obj),
            Err(_) => Err(AppError::new(message, error)),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.error {
            AppErrorKind::WrongMimeType => write!(f, "Unsupported MimeType: {}", self.message),
            AppErrorKind::InternalServerError => {
                write!(f, "Internal server error: {}", self.message)
            }
            AppErrorKind::NotFound => {
                write!(f, "Resource not found: {}", self.message)
            }
        }
    }
}

impl From<DatabaseError> for AppError {
    fn from(value: DatabaseError) -> Self {
        Self::new(&value.error, AppErrorKind::InternalServerError)
    }
}
