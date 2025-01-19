use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::models;
use crate::business::models::error::{AppError, AppErrorKind, MapToAppError};
use crate::business::models::video::{VideoEditReq, VideoUploadReq};
use crate::business::util::file::create_dir_if_not_exist;
use crate::business::validation::validatable::{EmptyContext, Validatable};
use crate::business::Result;
use crate::persistence::entities::video::{PatchVideo, Video, VideoVisibility};
use crate::persistence::repositories::unit_of_work::UnitOfWork;
use crate::persistence::repositories::video::VideoRepo;
use actix_files::NamedFile;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait VideoFacadeTrait {
    async fn save_video(&self, user_id: i32, video: VideoUploadReq)
        -> Result<models::video::Video>;
    async fn patch_video(
        &self,
        user_id: i32,
        video_id: i32,
        video: VideoEditReq,
    ) -> Result<models::video::Video>;
    async fn delete_video(&self, user_id: i32, video_id: i32) -> Result<()>;
    async fn get_video_entity(&self, video_id: i32, user_id: i32) -> Result<Video>;
    async fn get_video_model(&self, video_id: i32, user_id: i32) -> Result<models::video::Video>;
    async fn get_playable_video(&self, video_id: i32, user_id: i32) -> Result<NamedFile>;
    async fn get_thumbnail_file(&self, video_id: i32, user_id: i32) -> Result<NamedFile>;
    fn get_video_thumbnail_dirs(&self) -> (String, String);
}

#[derive(Clone)]
pub struct VideoFacade {
    temp_file_facade: Arc<TempFileFacade>,
    video_repo: Arc<dyn VideoRepo + Sync + Send>,
    unit_of_work: Arc<dyn UnitOfWork + Sync + Send>,
    video_dir: String,
    thumbnail_dir: String,
}

impl VideoFacade {
    pub fn new(
        temp_file_facade: Arc<TempFileFacade>,
        video_repo: Arc<dyn VideoRepo + Sync + Send>,
        unit_of_work: Arc<dyn UnitOfWork + Sync + Send>,
        video_dir: String,
        thumbnail_dir: String,
    ) -> Self {
        Self {
            temp_file_facade,
            video_repo,
            unit_of_work,
            video_dir,
            thumbnail_dir,
        }
    }

    pub async fn create_dirs(video_path: String, thumbnail_path: String) -> anyhow::Result<()> {
        create_dir_if_not_exist(video_path).await?;
        create_dir_if_not_exist(thumbnail_path).await?;

        Ok(())
    }
}

#[async_trait]
impl VideoFacadeTrait for VideoFacade {
    /// Permanently saves video with the given attributes
    ///
    /// This function calls [`business::facades::temp_file`] service to store temporary files
    /// permanently on the given location.
    ///
    ///
    /// # Arguments
    ///
    /// * `user_id` - ID of an artist which want to save the video.
    /// * `video_model` - Includes IDs of temporary files and needed metadata to correctly store
    /// the video
    async fn save_video(
        &self,
        user_id: i32,
        video_model: VideoUploadReq,
    ) -> Result<models::video::Video> {
        video_model
            .validate_model(&EmptyContext::new())
            .await
            .app_error_kind("Validation failed", AppErrorKind::BadRequestError)?;

        let mut tx = self.unit_of_work.begin().await?;
        let video_path = self
            .temp_file_facade
            .persist_permanently(
                video_model.temp_video_id,
                user_id,
                self.video_dir.clone(),
                &mut tx,
            )
            .await?;
        let thumbnail_path = self
            .temp_file_facade
            .persist_permanently(
                video_model.temp_thumbnail_id,
                user_id,
                self.thumbnail_dir.clone(),
                &mut tx,
            )
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

        let video_entity = self.video_repo.save_video(entity, &mut tx).await?;

        tx.commit().await.app_error("Failed save the video")?;

        Ok(models::video::Video::from(&video_entity))
    }

    async fn patch_video(
        &self,
        user_id: i32,
        video_id: i32,
        edited_video: VideoEditReq,
    ) -> Result<models::video::Video> {
        let mut tx = self.unit_of_work.begin().await?;

        let video_path = if let Some(file_id) = edited_video.temp_video_id {
            let path = self
                .temp_file_facade
                .persist_permanently(file_id, user_id, self.video_dir.clone(), &mut tx)
                .await?;
            Some(path)
        } else {
            None
        };

        let thumbnail_path = if let Some(temp_thumbnail) = edited_video.temp_thumbnail_id {
            let path = self
                .temp_file_facade
                .persist_permanently(temp_thumbnail, user_id, self.thumbnail_dir.clone(), &mut tx)
                .await?;
            Some(path)
        } else {
            None
        };
        let db_entity = PatchVideo {
            id: video_id,
            artist_id: None,
            visibility: VideoVisibility::from(&edited_video.video_visibility),
            name: edited_video.name,
            file_path: video_path,
            thumbnail_path,
            description: edited_video.description,
        };

        let video = self.video_repo.patch_video(db_entity, &mut tx).await?;

        tx.commit().await.app_error("Failed to patch the video")?;

        Ok(models::video::Video::from(&video))
    }

    async fn delete_video(&self, user_id: i32, video_id: i32) -> Result<()> {
        let deleted = self.video_repo.delete_video(video_id, user_id).await?;
        if !deleted {
            return Err(AppError::new("Video doesn't exist", AppErrorKind::NotFound));
        }

        Ok(())
    }

    /// Returns video model by given video_id for rendering in the template.
    ///
    /// TODO: Check if user can access the video!
    ///
    /// # Arguments
    ///
    /// * `video_id` - ID of the video you want to get
    /// * `user_id` - ID of an user that requested the video
    async fn get_video_entity(&self, video_id: i32, _user_id: i32) -> Result<Video> {
        let video_entity = self
            .video_repo
            .get_video_by_id(video_id)
            .await?
            .ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;
        Ok(video_entity)
    }

    async fn get_video_model(&self, video_id: i32, _user_id: i32) -> Result<models::video::Video> {
        let video_entity = self
            .video_repo
            .get_video_by_id(video_id)
            .await?
            .ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;
        Ok(models::video::Video::from(&video_entity))
    }

    /// Serves directly video file for video player
    ///
    /// TODO: Check if user can access the video!
    ///
    /// * `video_id` - ID of the video you want to get
    /// * `user_id` - ID of an user that requested the video
    async fn get_playable_video(&self, video_id: i32, _user_id: i32) -> Result<NamedFile> {
        let video_entity = self
            .video_repo
            .get_video_by_id(video_id)
            .await?
            .ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;
        let path = Path::new(video_entity.file_path.as_str());
        let file = NamedFile::open_async(path)
            .await
            .app_error("Video doesn't exist")?;

        Ok(file)
    }

    /// Returns thumbnail image directly to the client
    ///
    /// TODO: Check if user can access the video!
    ///
    /// * `video_id` - ID of the video you want to get
    /// * `user_id` - ID of an user that requested the video
    async fn get_thumbnail_file(&self, video_id: i32, _user_id: i32) -> Result<NamedFile> {
        let video_entity = self
            .video_repo
            .get_video_by_id(video_id)
            .await?
            .ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;
        let path = Path::new(video_entity.thumbnail_path.as_str());
        let file = NamedFile::open_async(path)
            .await
            .app_error("Thumbnail doesn't exist")?;

        Ok(file)
    }

    fn get_video_thumbnail_dirs(&self) -> (String, String) {
        (self.video_dir.clone(), self.thumbnail_dir.clone())
    }
}
