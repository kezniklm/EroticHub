use crate::business::models::video_category::VideoCategory;
use crate::persistence::entities::video_category::VideoCategory as VideoCategoryEntity;

impl From<VideoCategoryEntity> for VideoCategory {
    fn from(category: VideoCategoryEntity) -> Self {
        VideoCategory {
            id: category.id,
            name: category.name,
        }
    }
}
