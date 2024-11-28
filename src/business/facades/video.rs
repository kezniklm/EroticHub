use crate::business::facades::temp_file::TempFileFacade;
use crate::business::models::video::VideoUploadData;
use crate::persistence::repositories::video::VideoRepo;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait VideoFacadeTrait {
    async fn save_video(&self, user_id: i32, video: VideoUploadData) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct VideoFacade {
    temp_file_facade: Arc<TempFileFacade>,
    video_repo: Arc<dyn VideoRepo + Sync + Send>,
}

impl VideoFacade {
    pub fn new(
        temp_file_facade: Arc<TempFileFacade>,
        video_repo: Arc<dyn VideoRepo + Sync + Send>,
    ) -> Self {
        Self {
            temp_file_facade,
            video_repo,
        }
    }
}

#[async_trait]
impl VideoFacadeTrait for VideoFacade {
    async fn save_video(&self, user_id: i32, video: VideoUploadData) -> anyhow::Result<()> {
        // self.temp_file_facade
        todo!();
    }
}
