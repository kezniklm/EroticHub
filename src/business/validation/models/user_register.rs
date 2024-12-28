use crate::business::models::user_register::UserRegister;
use crate::business::validation::contexts::user::UserValidationContext;
use crate::business::validation::utils::extract_first_error;
use crate::business::validation::validatable::Validatable;
use validator::{Validate, ValidationError};

impl Validatable<UserValidationContext> for UserRegister {
    async fn validate_model(&self, context: &UserValidationContext) -> Result<(), ValidationError> {
        if let Ok(Some(_)) = context
            .user_repository
            .as_ref()
            .get_user_by_username(&self.username)
            .await
        {
            let mut error = ValidationError::new("username_already_exists");
            error.message = Some("The username is already taken".into());
            return Err(error);
        }

        if let Err(validation_errors) = self.validate() {
            if let Some(first_error) = extract_first_error(&validation_errors) {
                return Err(first_error);
            }
        }

        Ok(())
    }
}
