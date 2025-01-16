use crate::business::models::stream::{
    LiveStream as LiveStreamDto, LiveStreamStart, LiveStreamStatus as StatusDto,
};
use crate::persistence::entities::stream::{
    LiveStream as LiveStreamEntity, LiveStreamStatus as StatusEntity,
};

impl From<&LiveStreamStart> for LiveStreamEntity {
    fn from(value: &LiveStreamStart) -> Self {
        Self {
            id: -1,
            video_id: value.video_id,
            start_time: chrono::Local::now(),
            status: StatusEntity::Running,
        }
    }
}

impl From<StatusEntity> for StatusDto {
    fn from(value: StatusEntity) -> Self {
        match value {
            StatusEntity::Pending => StatusDto::Pending,
            StatusEntity::Running => StatusDto::Running,
            StatusEntity::Ended => StatusDto::Ended,
        }
    }
}

impl LiveStreamDto {
    pub(crate) fn from_entity(value: LiveStreamEntity, stream_url: String) -> Self {
        Self {
            id: value.id,
            video_id: value.video_id,
            start_time: value.start_time,
            status: value.status.into(),
            stream_url,
        }
    }
}
