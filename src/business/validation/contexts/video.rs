use crate::persistence::repositories::user::UserRepositoryTrait;
use std::sync::Arc;

pub struct VideoValidationContext {
    pg_video_repo: Arc<dyn UserRepositoryTrait + Send + Sync>,
}
