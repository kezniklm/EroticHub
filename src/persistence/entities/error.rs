use log::error;
use sqlx::Error;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub struct DatabaseError {
    pub error: String,
}

impl DatabaseError {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl From<Error> for DatabaseError {
    fn from(value: Error) -> Self {
        DatabaseError::new(value.to_string())
    }
}

/// Conversion to be used only in the tests
/// We don't want to expose all messages from the io Results to clients, since
/// it can contain sensitive information
#[cfg(test)]
impl From<std::io::Error> for DatabaseError {
    fn from(value: std::io::Error) -> Self {
        DatabaseError::new(value.to_string())
    }
}

pub trait MapToDatabaseError<T> {
    /// If result contains Error, it is mapped to `DatabaseError` with given message. \
    /// Ok result is not touched.
    fn db_error(self, message: &str) -> Result<T, DatabaseError>;
}

impl<T, E: Debug> MapToDatabaseError<T> for Result<T, E> {
    fn db_error(self, message: &str) -> Result<T, DatabaseError> {
        match self {
            Ok(obj) => Ok(obj),
            Err(err) => {
                error!("{:#?}", err);
                Err(DatabaseError::new(message.to_string()))
            }
        }
    }
}
