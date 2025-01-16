use validator::ValidationError;

pub trait Validatable<T> {
    async fn validate_model(&self, context: &T) -> Result<(), ValidationError>;
}
