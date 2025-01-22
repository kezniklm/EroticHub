use crate::business::models::error::{AppError, AppErrorKind, MapToAppError};
use crate::business::util::file::{create_dir_if_not_exist, get_file_extension};
use crate::business::Result;
use crate::persistence::entities::temp_file::TempFile;
use crate::persistence::repositories::temp_file::TempFileRepo;
use actix_files::NamedFile;
use async_trait::async_trait;
use log::{debug, warn};
use sqlx::{Postgres, Transaction};
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;
use uuid::Uuid;

#[async_trait]
pub trait TempFileFacadeTrait {
    async fn persist_temp_file(
        &self,
        temp_file: NamedTempFile,
        file_name: String,
        user_id: i32,
    ) -> Result<i32>;

    async fn get_temp_file(&self, file_id: i32, user_id: i32) -> Result<NamedFile>;
    async fn create_temp_directory(temp_file_dir: String) -> anyhow::Result<()>;
    async fn delete_all_temp_files(&self) -> Result<()>;
    async fn check_mime_type(&self, file: Option<String>, allowed_types: Vec<String>)
        -> Result<()>;

    async fn persist_permanently(
        &self,
        file_id: i32,
        user_id: i32,
        path: String,
        tx: &mut Transaction<Postgres>,
    ) -> Result<String>;
    async fn delete_temp_file(&self, temp_file_id: i32, user_id: i32) -> Result<()>;
    fn get_temp_directory_path(&self) -> String;
}

#[derive(Clone)]
pub struct TempFileFacade {
    temp_file_repo: Arc<dyn TempFileRepo + Sync + Send>,
    temp_file_dir: String,
}

impl TempFileFacade {
    pub fn new(temp_file_repo: Arc<dyn TempFileRepo + Sync + Send>, temp_file_dir: String) -> Self {
        Self {
            temp_file_repo,
            temp_file_dir,
        }
    }
}

#[async_trait]
impl TempFileFacadeTrait for TempFileFacade {
    /// Persists temporary file
    ///
    /// # Arguments
    /// * `temp_file` - representation of the temporary file received through an endpoint
    /// * `file_name` - name of the file including extension received through an endpoint
    /// * `user_id` - ID of the user that performed the action
    ///
    /// # Returns
    ///
    /// * `Temp file ID` - ID of temporary file, which can be sent back to client,
    /// and later used for requesting the temporary file.
    async fn persist_temp_file(
        &self,
        temp_file: NamedTempFile,
        file_name: String,
        user_id: i32,
    ) -> Result<i32> {
        let uuid = Uuid::new_v4();

        let path_str = format!(
            "./{}/{}.{}",
            self.temp_file_dir,
            uuid,
            get_file_extension(file_name).await
        );
        let entity = TempFile {
            id: -1,
            user_id,
            file_path: path_str.clone(),
        };

        let temp_file_id = self.temp_file_repo.add_file(entity, temp_file).await?;

        debug!(
            "Stored temp file with ID: {} and path: {}",
            &temp_file_id, &path_str
        );

        Ok(temp_file_id)
    }

    async fn get_temp_file(&self, file_id: i32, user_id: i32) -> Result<NamedFile> {
        let temp_file = self
            .temp_file_repo
            .get_file(file_id, user_id)
            .await
            .app_error_kind("Temp file doesn't exist", AppErrorKind::NotFound)?;

        let path = Path::new(temp_file.file_path.as_str());
        let file = NamedFile::open_async(path)
            .await
            .app_error_kind("Temporary file not found", AppErrorKind::NotFound)?;

        Ok(file)
    }

    async fn create_temp_directory(temp_file_dir: String) -> anyhow::Result<()> {
        create_dir_if_not_exist(temp_file_dir).await?;
        Ok(())
    }

    async fn delete_all_temp_files(&self) -> Result<()> {
        let temp_dir_path = Path::new(self.temp_file_dir.as_str());
        if !temp_dir_path.exists() {
            return Ok(());
        }
        self.temp_file_repo
            .delete_all_files(temp_dir_path)
            .await
            .app_error("Failed to create temp file directory")?;

        debug!("All temp files were deleted!");
        Ok(())
    }

    async fn check_mime_type(
        &self,
        file: Option<String>,
        allowed_types: Vec<String>,
    ) -> Result<()> {
        if let Some(ref mime_type) = file {
            let is_allowed = allowed_types.contains(mime_type);
            if is_allowed {
                return Ok(());
            }

            warn!(
                "File with unsupported MimeType '{}' was uploaded!",
                mime_type
            );
        }

        Err(AppError::new(
            &format!("MimeType {:?} is not allowed", file),
            AppErrorKind::WrongMimeType,
        ))
    }

    /// Persists temporary file permanently to the given location
    ///
    /// Function removes the temporary file from the database and moves the actual file to the
    /// permanent storage on file system. For security reason, underlying request to repository
    /// checks if the user_id matches with user which created the temporary file. If not, Error is
    /// returned.
    ///
    /// # Arguments
    ///
    /// * `file_id` - ID of the temporary file.
    /// * `user_id` - ID of the user who performed the store action.
    /// * `permanent_path` - String with the path where to move the temporary file.
    ///
    /// # Returns
    ///
    /// - new path to the saved file as String.
    async fn persist_permanently(
        &self,
        file_id: i32,
        user_id: i32,
        permanent_path: String,
        tx: &mut Transaction<Postgres>,
    ) -> Result<String> {
        let temp_file = self
            .temp_file_repo
            .get_file_tx(file_id, user_id, tx)
            .await
            .app_error_kind("Video file doesn't exist", AppErrorKind::NotFound)?;

        let temp_file_path = Path::new(temp_file.file_path.as_str());

        let new_path = format!(
            "{permanent_path}/{}",
            temp_file_path.file_name().unwrap().to_str().unwrap_or("")
        );
        println!("{:?}", new_path);
        tokio::fs::rename(temp_file_path, &new_path)
            .await
            .app_error("Operation with temp file failed")?;

        self.temp_file_repo
            .delete_file(file_id, user_id)
            .await
            .app_error("Failed to delete temp file")?;
        Ok(new_path)
    }

    async fn delete_temp_file(&self, temp_file_id: i32, user_id: i32) -> Result<()> {
        self.temp_file_repo
            .delete_file(temp_file_id, user_id)
            .await?;

        Ok(())
    }

    fn get_temp_directory_path(&self) -> String {
        self.temp_file_dir.clone()
    }
}
