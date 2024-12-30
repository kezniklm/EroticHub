use crate::business::models::video::VideoVisibility as VideoVisibilityDto;
use crate::business::models::video::{EditVideoTemplateModel, Video as VideoDto};
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

impl From<EditVideoTemplateModel> for VideoDto {
    fn from(value: EditVideoTemplateModel) -> Self {
        VideoDto {
            id: value.id,
            artist_id: -1,
            video_visibility: value.video_visibility,
            name: value.name,
            description: Option::from(value.description),
        }
    }
}

impl From<VideoDto> for EditVideoTemplateModel {
    fn from(value: VideoDto) -> Self {
        EditVideoTemplateModel {
            id: value.id,
            video_visibility: value.video_visibility,
            name: value.name,
            description: value.description.unwrap_or(String::from("")),
        }
    }
}
