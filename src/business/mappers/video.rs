use crate::business::models::video::VideoVisibility as VideoVisibilityDto;
use crate::persistence::entities::video::VideoVisibility as VideoVisibilityEntity;

impl From<&VideoVisibilityDto> for VideoVisibilityEntity {
    fn from(value: &VideoVisibilityDto) -> Self {
        match value {
            VideoVisibilityDto::ALL => VideoVisibilityEntity::ALL,
            VideoVisibilityDto::REGISTERED => VideoVisibilityEntity::REGISTERED,
            VideoVisibilityDto::PAYING => VideoVisibilityEntity::PAYING,
        }
    }
}
