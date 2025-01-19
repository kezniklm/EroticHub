use crate::business::models::video::VideoUploadReq;
use crate::business::validation::utils::extract_first_error;
use crate::business::validation::validatable::{EmptyContext, Validatable};
use validator::{Validate, ValidationError};

impl Validatable<EmptyContext> for VideoUploadReq {
    async fn validate_model(&self, _context: &EmptyContext) -> Result<(), ValidationError> {
        if let Err(validation_errors) = self.validate() {
            if let Some(first_error) = extract_first_error(&validation_errors) {
                return Err(first_error);
            }
        }

        Ok(())
    }
}
