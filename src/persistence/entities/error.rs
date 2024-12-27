use std::fmt::Debug;
use log::error;
use sqlx::Error;

pub struct DatabaseError {
    pub error: String,
}

impl DatabaseError {
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

impl From<Error> for DatabaseError {
    fn from(value: Error) -> Self {
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
            },
        }
    }
}
