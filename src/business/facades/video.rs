use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::models::video::VideoUploadData;
use crate::business::util::file::create_dir_if_not_exist;
use crate::persistence::entities::video::{Video, VideoVisibility};
use crate::persistence::repositories::video::VideoRepo;
use async_trait::async_trait;
use std::sync::Arc;

const DEFAULT_VIDEO_DIRECTORY: &str = "./resources/videos";
const DEFAULT_THUMBNAILS_PATH: &str = "./resources/thumbnails";
const VIDEOS_DIRECTORY_KEY: &str = "VIDEO_DIRECTORY_PATH";
const THUMBNAIL_DIRECTORY_KEY: &str = "THUMBNAIL_DIRECTORY_PATH";

#[async_trait]
pub trait VideoFacadeTrait {
    async fn save_video(&self, user_id: i32, video: VideoUploadData) -> anyhow::Result<()>;
    fn get_video_thumbnail_dirs(&self) -> (String, String);
    async fn create_dirs(&self) -> anyhow::Result<()>;
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
    /// Permanently saves video with the given attributes
    ///
    /// This function calls [`business::facades::temp_file`] service to store temporary files
    /// permanently on the given location.
    ///
    /// # TODO
    /// Facade has to check if user (represented by user_id) is an artist and can save videos.
    ///
    /// # Arguments
    ///
    /// * `user_id` - ID of an artist which want to save the video.
    /// * `video_model` - Includes IDs of temporary files and needed metadata to correctly store
    /// the video
    async fn save_video(&self, user_id: i32, video_model: VideoUploadData) -> anyhow::Result<()> {
        let (video_dir_path, thumbnail_dir_path) = self.get_video_thumbnail_dirs();

        let video_path = self
            .temp_file_facade
            .persist_permanently(video_model.temp_video_id, user_id, video_dir_path)
            .await?;
        let thumbnail_path = self
            .temp_file_facade
            .persist_permanently(video_model.temp_thumbnail_id, user_id, thumbnail_dir_path)
            .await?;

        let entity = Video {
            id: -1,
            artist_id: user_id,
            visibility: VideoVisibility::from(&video_model.video_visibility),
            name: video_model.name,
            file_path: video_path,
            thumbnail_path,
            description: video_model.description,
        };

        self.video_repo.save_video(entity).await?;

        Ok(())
    }

    /// Function returns path to both video and thumbnail folder, where the files are stored.
    ///
    /// # Returns
    ///
    /// Tuple with:
    /// - Path to video directory as String
    /// - Path to thumbnails directory as String
    fn get_video_thumbnail_dirs(&self) -> (String, String) {
        let video =
            dotenvy::var(VIDEOS_DIRECTORY_KEY).unwrap_or(DEFAULT_VIDEO_DIRECTORY.to_string());
        let thumbnail =
            dotenvy::var(THUMBNAIL_DIRECTORY_KEY).unwrap_or(DEFAULT_THUMBNAILS_PATH.to_string());
        (video, thumbnail)
    }

    async fn create_dirs(&self) -> anyhow::Result<()> {
        let (video_path, thumbnail_path) = self.get_video_thumbnail_dirs();
        create_dir_if_not_exist(video_path).await?;
        create_dir_if_not_exist(thumbnail_path).await?;

        Ok(())
    }
}
