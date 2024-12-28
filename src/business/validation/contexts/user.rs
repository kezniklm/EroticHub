use crate::persistence::repositories::user::UserRepositoryTrait;
use std::sync::Arc;

pub struct UserValidationContext {
    pub user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,
}
