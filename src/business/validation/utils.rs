use validator::{ValidationError, ValidationErrors};

pub fn extract_first_error(errors: &ValidationErrors) -> Option<ValidationError> {
    for error_kind in errors.errors().values() {
        if let validator::ValidationErrorsKind::Field(field_errors) = error_kind {
            if let Some(first_error) = field_errors.first() {
                return Some(first_error.clone());
            }
        }
    }
    None
}
