use crate::persistence::entities::error::DatabaseError;

pub mod repositories;

pub mod entities;

pub type Result<T, E = DatabaseError> = core::result::Result<T, E>;
