use crate::business::util::file::create_dir_if_not_exist;
use crate::persistence::entities::temp_file::TempFile;
use crate::persistence::repositories::temp_file::TempFileRepo;
use actix_files::NamedFile;
use anyhow::Error;
use async_trait::async_trait;
use log::{debug, warn};
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;
use uuid::Uuid;

const DEFAULT_TEMP_DIRECTORY: &str = "temp";
const TEMP_DIRECTORY_KEY: &str = "TEMP_DIRECTORY_PATH";
#[async_trait]
pub trait TempFileFacadeTrait {
    async fn persist_temp_file(
        &self,
        temp_file: NamedTempFile,
        file_name: String,
        user_id: i32,
    ) -> anyhow::Result<i32>;

    fn get_temp_directory_path(&self) -> String;
    async fn get_temp_file(&self, file_id: i32, user_id: i32) -> anyhow::Result<NamedFile>;
    async fn create_temp_directory(&self) -> anyhow::Result<()>;
    async fn delete_all_temp_files(&self) -> anyhow::Result<()>;
    async fn delete_temp_file(&self, temp_file_id: i32, user_id: i32) -> anyhow::Result<()>;
    async fn check_mime_type(
        &self,
        file: Option<String>,
        allowed_types: Vec<String>,
    ) -> anyhow::Result<()>;

    async fn persist_permanently(
        &self,
        file_id: i32,
        user_id: i32,
        path: String,
    ) -> anyhow::Result<String>;
}

#[derive(Clone)]
pub struct TempFileFacade {
    temp_file_repo: Arc<dyn TempFileRepo + Sync + Send>,
}

impl TempFileFacade {
    pub fn new(temp_file_repo: Arc<dyn TempFileRepo + Sync + Send>) -> Self {
        Self { temp_file_repo }
    }

    fn get_file_extension(&self, file_name: String) -> String {
        if let Some(file_name) = file_name.split_once(".") {
            let (_name, extension) = file_name;
            return extension.to_string();
        }
        String::new()
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
    ) -> anyhow::Result<i32> {
        let uuid = Uuid::new_v4();

        let path_str = format!(
            "./{}/{}.{}",
            self.get_temp_directory_path(),
            uuid,
            self.get_file_extension(file_name)
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

    fn get_temp_directory_path(&self) -> String {
        dotenvy::var(TEMP_DIRECTORY_KEY).unwrap_or(DEFAULT_TEMP_DIRECTORY.to_string())
    }

    async fn get_temp_file(&self, file_id: i32, user_id: i32) -> anyhow::Result<NamedFile> {
        let temp_file = self
            .temp_file_repo
            .get_file(file_id, user_id)
            .await?
            .ok_or(Error::msg("Temp file doesn't exist"))?;
        let path = Path::new(temp_file.file_path.as_str());
        let file = NamedFile::open_async(path).await?;

        Ok(file)
    }

    async fn create_temp_directory(&self) -> anyhow::Result<()> {
        let temp_directory = self.get_temp_directory_path();
        create_dir_if_not_exist(temp_directory).await?;
        Ok(())
    }

    async fn delete_all_temp_files(&self) -> anyhow::Result<()> {
        let temp_dir_path = self.get_temp_directory_path();
        let temp_dir_path = Path::new(temp_dir_path.as_str());
        if !temp_dir_path.exists() {
            return Ok(());
        }
        self.temp_file_repo.delete_all_files(temp_dir_path).await?;

        debug!("All temp files were deleted!");
        Ok(())
    }

    async fn delete_temp_file(&self, temp_file_id: i32, user_id: i32) -> anyhow::Result<()> {
        self.temp_file_repo
            .delete_file(temp_file_id, user_id)
            .await?;

        Ok(())
    }

    async fn check_mime_type(
        &self,
        file: Option<String>,
        allowed_types: Vec<String>,
    ) -> anyhow::Result<()> {
        if let Some(mime_type) = file {
            let is_allowed = allowed_types.contains(&mime_type);
            if is_allowed {
                return Ok(());
            }

            warn!(
                "File with unsupported MimeType '{}' was uploaded!",
                mime_type
            );
        }

        Err(Error::msg("Unsupported MimeType"))
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
    ) -> anyhow::Result<String> {
        let temp_file = self
            .temp_file_repo
            .get_file(file_id, user_id)
            .await?
            .ok_or(Error::msg("Temporary file doesn't exist"))?;

        let temp_file_path = Path::new(temp_file.file_path.as_str());

        let new_path = format!(
            "{permanent_path}/{}",
            temp_file_path.file_name().unwrap().to_str().unwrap_or("")
        );
        println!("{:?}", new_path);
        tokio::fs::rename(temp_file_path, &new_path).await?;

        self.temp_file_repo.delete_file(file_id, user_id).await?;
        Ok(new_path)
    }
}
