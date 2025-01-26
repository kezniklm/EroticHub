use crate::business::facades::artist::ArtistFacadeTrait;
use crate::business::facades::temp_file::{TempFileFacade, TempFileFacadeTrait};
use crate::business::facades::user::UserFacadeTrait;
use crate::business::models;
use crate::business::models::error::{AppError, AppErrorKind, MapToAppError};
use crate::business::models::user::UserRole;
use crate::business::models::video::{VideoEditReq, VideoUploadReq};
use crate::business::util::file::create_dir_if_not_exist;
use crate::business::validation::contexts::video::PatchVideoValidationContext;
use crate::business::validation::validatable::{EmptyContext, Validatable};
use crate::business::Result;
use crate::persistence::entities::video::{PatchVideo, Video, VideoVisibility};
use crate::persistence::repositories::unit_of_work::UnitOfWork;
use crate::persistence::repositories::video::VideoRepo;
use actix_files::NamedFile;
use async_trait::async_trait;
use std::collections::HashSet;
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
    async fn get_video_entity(&self, video_id: i32, user_id: Option<i32>) -> Result<Video>;
    async fn get_video_model(
        &self,
        video_id: i32,
        user_id: Option<i32>,
    ) -> Result<models::video::Video>;
    async fn get_playable_video(&self, video_id: i32, user_id: Option<i32>) -> Result<NamedFile>;
    async fn get_thumbnail_file(&self, video_id: i32, user_id: Option<i32>) -> Result<NamedFile>;
    async fn check_permissions(&self, video: &Video, user_id: Option<i32>) -> Result<()>;
    async fn get_video_list(&self) -> Result<Vec<Video>>;
    async fn fetch_videos(
        &self,
        ord: Option<&str>,
        filter: Option<Vec<i32>>,
        offset: Option<i32>,
    ) -> Result<Vec<Video>>;
    fn get_video_thumbnail_dirs(&self) -> (String, String);
}

#[derive(Clone)]
pub struct VideoFacade {
    temp_file_facade: Arc<TempFileFacade>,
    video_repo: Arc<dyn VideoRepo + Sync + Send>,
    artist_facade: Arc<dyn ArtistFacadeTrait + Sync + Send>,
    user_facade: Arc<dyn UserFacadeTrait + Sync + Send>,
    unit_of_work: Arc<dyn UnitOfWork + Sync + Send>,
    video_dir: String,
    thumbnail_dir: String,
}

impl VideoFacade {
    pub fn new(
        temp_file_facade: Arc<TempFileFacade>,
        video_repo: Arc<dyn VideoRepo + Sync + Send>,
        artist_facade: Arc<dyn ArtistFacadeTrait + Sync + Send>,
        user_facade: Arc<dyn UserFacadeTrait + Sync + Send>,
        unit_of_work: Arc<dyn UnitOfWork + Sync + Send>,
        video_dir: String,
        thumbnail_dir: String,
    ) -> Self {
        Self {
            temp_file_facade,
            video_repo,
            artist_facade,
            user_facade,
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
        let mut tx = self.unit_of_work.begin().await?;
        let artist = self
            .artist_facade
            .get_artist_internal(user_id, Some(&mut tx))
            .await?;

        video_model
            .validate_model(&EmptyContext::new())
            .await
            .app_error_kind("Validation failed", AppErrorKind::BadRequestError)?;

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
            artist_id: artist.id,
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
        let context = PatchVideoValidationContext {
            pg_video_repo: self.video_repo.clone(),
            artist_facade: self.artist_facade.clone(),
            user_id,
            video_id,
        };
        edited_video
            .validate_model(&context)
            .await
            .app_error_kind("Validation failed", AppErrorKind::BadRequestError)?;

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
        let mut tx = self.unit_of_work.begin().await?;

        let artist = self
            .artist_facade
            .get_artist_internal(user_id, Some(&mut tx))
            .await?;
        let deleted = self
            .video_repo
            .delete_video(video_id, artist.id, &mut tx)
            .await?;

        deleted.ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;

        tx.commit().await.app_error("Failed to delete the video")?;

        Ok(())
    }

    /// Returns video model by given video_id for rendering in the template.
    ///
    /// # Arguments
    ///
    /// * `video_id` - ID of the video you want to get
    /// * `user_id` - ID of an user that requested the video
    async fn get_video_entity(&self, video_id: i32, user_id: Option<i32>) -> Result<Video> {
        let video_entity = self
            .video_repo
            .get_video_by_id(video_id, None)
            .await?
            .ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;

        self.check_permissions(&video_entity, user_id).await?;
        Ok(video_entity)
    }

    async fn get_video_model(
        &self,
        video_id: i32,
        user_id: Option<i32>,
    ) -> Result<models::video::Video> {
        let video_entity = self
            .video_repo
            .get_video_by_id(video_id, None)
            .await?
            .ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;

        self.check_permissions(&video_entity, user_id).await?;
        Ok(models::video::Video::from(&video_entity))
    }

    /// Serves directly video file for video player
    ///
    /// * `video_id` - ID of the video you want to get
    /// * `user_id` - ID of an user that requested the video
    async fn get_playable_video(&self, video_id: i32, user_id: Option<i32>) -> Result<NamedFile> {
        let video_entity = self
            .video_repo
            .get_video_by_id(video_id, None)
            .await?
            .ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;

        self.check_permissions(&video_entity, user_id).await?;

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
    async fn get_thumbnail_file(&self, video_id: i32, user_id: Option<i32>) -> Result<NamedFile> {
        let video_entity = self
            .video_repo
            .get_video_by_id(video_id, None)
            .await?
            .ok_or(AppError::new("Video doesn't exist", AppErrorKind::NotFound))?;

        self.check_permissions(&video_entity, user_id).await?;
        let path = Path::new(video_entity.thumbnail_path.as_str());
        let file = NamedFile::open_async(path)
            .await
            .app_error("Thumbnail doesn't exist")?;

        Ok(file)
    }

    async fn check_permissions(&self, video: &Video, user_id: Option<i32>) -> Result<()> {
        let permissions = match user_id {
            None => HashSet::new(),
            Some(user_id) => self.user_facade.get_permissions(user_id).await?,
        };

        if permissions.contains(&UserRole::Artist) {
            let artist = self
                .artist_facade
                .get_artist_internal(user_id.unwrap(), None)
                .await?; // if permissions hashset contains any UserRole, user_id is always Some

            // Artist has always access to his video
            if artist.id == video.artist_id {
                return Ok(());
            }
        }

        let check = |role: &UserRole| -> Result<()> {
            if permissions.contains(role) {
                return Ok(());
            }
            Err(AppError::new("Video doesn't exist", AppErrorKind::NotFound))
        };
        match video.visibility {
            VideoVisibility::All => Ok(()),
            VideoVisibility::Registered => check(&UserRole::Registered),
            VideoVisibility::Paying => check(&UserRole::PayingMember),
        }
    }

    async fn get_video_list(&self) -> Result<Vec<Video>> {
        let videos = self.video_repo.list_videos().await;
        let videos = match videos {
            Ok(videos) => videos,
            Err(_e) => {
                return Err(AppError::new(
                    "Error fetching videos",
                    AppErrorKind::InternalServerError,
                ))
            }
        };
        Ok(videos)
    }

    async fn fetch_videos(
        &self,
        ord: Option<&str>,
        filter: Option<Vec<i32>>,
        offset: Option<i32>,
    ) -> Result<Vec<Video>> {
        let videos = self.video_repo.fetch_videos(ord, filter, offset).await;
        let videos = match videos {
            Ok(videos) => videos,
            Err(_e) => {
                return Err(AppError::new(
                    "Error fetching videos",
                    AppErrorKind::InternalServerError,
                ))
            }
        };
        Ok(videos)
    }

    fn get_video_thumbnail_dirs(&self) -> (String, String) {
        (self.video_dir.clone(), self.thumbnail_dir.clone())
    }
}
