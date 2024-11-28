use crate::business::models::video::VideoUploadData as VideoUploadDataDto;
use crate::persistence::entities::video::Video as VideoEntity;

impl From<VideoUploadDataDto> for VideoEntity {
    fn from(value: VideoUploadDataDto) -> Self {
        // VideoEntity { }
        todo!();
    }
}
