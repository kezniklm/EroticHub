use crate::business::models::video::Video as VideoDto;
use crate::business::models::video::VideoVisibility as VideoVisibilityDto;
use crate::persistence::entities::video::Video as VideoEntity;
use crate::persistence::entities::video::VideoVisibility as VideoVisibilityEntity;

impl From<&VideoVisibilityDto> for VideoVisibilityEntity {
    fn from(value: &VideoVisibilityDto) -> Self {
        match value {
            VideoVisibilityDto::All => VideoVisibilityEntity::All,
            VideoVisibilityDto::Registered => VideoVisibilityEntity::Registered,
            VideoVisibilityDto::Paying => VideoVisibilityEntity::Paying,
        }
    }
}

impl From<&VideoVisibilityEntity> for VideoVisibilityDto {
    fn from(value: &VideoVisibilityEntity) -> Self {
        match value {
            VideoVisibilityEntity::All => VideoVisibilityDto::All,
            VideoVisibilityEntity::Registered => VideoVisibilityDto::Registered,
            VideoVisibilityEntity::Paying => VideoVisibilityDto::Paying,
        }
    }
}

impl From<&VideoEntity> for VideoDto {
    fn from(value: &VideoEntity) -> Self {
        Self {
            id: value.id,
            artist_id: value.artist_id,
            video_visibility: VideoVisibilityDto::from(&value.visibility),
            name: value.name.clone(),
            description: value.description.clone(),
        }
    }
}
