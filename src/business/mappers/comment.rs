use crate::business::models::comment::CommentModel;
use crate::persistence::entities::comment::CommentEntity;

impl From<CommentEntity> for CommentModel {
    fn from(comment_entity: CommentEntity) -> Self {
        CommentModel {
            id: comment_entity.id,
            user_id: comment_entity.user_id,
            created_at: comment_entity.created_at,
            video_id: comment_entity.video_id,
            content: comment_entity.content,
        }
    }
}
