use crate::business::models::error::AppError;

pub mod models;

pub mod facades;

pub mod mappers;
mod util;

pub type Result<T, E = AppError> = core::result::Result<T, E>;
mod validation;
