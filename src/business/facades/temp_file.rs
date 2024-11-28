use crate::business::models::video::TempFileResponse;
use crate::persistence::entities::temp_file::TempFile;
use crate::persistence::repositories::temp_file::TempFileRepo;
use anyhow::Error;
use async_trait::async_trait;
use log::{debug, warn};
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::fs::create_dir;
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
    ) -> anyhow::Result<TempFileResponse>;
    /// Checks if temp file belongs to current user. Is so, stores it to given path
    async fn get_file_path(&self, user_id: i32, permanent_path: &Path);
    fn get_temp_directory_path(&self) -> String;
    async fn create_temp_directory(&self) -> anyhow::Result<()>;
    async fn delete_all_temp_files(&self) -> anyhow::Result<()>;
    async fn check_mime_type(
        &self,
        file: Option<String>,
        allowed_types: Vec<String>,
    ) -> anyhow::Result<()>;

    async fn persist_permanently(
        &self,
        file_id: i32,
        user_id: i32,
        path: &Path,
    ) -> anyhow::Result<()>;
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
    async fn persist_temp_file(
        &self,
        temp_file: NamedTempFile,
        file_name: String,
        user_id: i32,
    ) -> anyhow::Result<TempFileResponse> {
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
        let response = TempFileResponse { temp_file_id };
        Ok(response)
    }

    async fn get_file_path(&self, user_id: i32, permanent_path: &Path) {
        todo!()
    }

    fn get_temp_directory_path(&self) -> String {
        dotenvy::var(TEMP_DIRECTORY_KEY).unwrap_or(DEFAULT_TEMP_DIRECTORY.to_string())
    }

    async fn create_temp_directory(&self) -> anyhow::Result<()> {
        let temp_directory = self.get_temp_directory_path();
        let path = Path::new(temp_directory.as_str());
        if path.exists() {
            return Ok(());
        }
        create_dir(path).await?;
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

    async fn persist_permanently(
        &self,
        file_id: i32,
        user_id: i32,
        new_path: &Path,
    ) -> anyhow::Result<()> {
        let temp_file = self
            .temp_file_repo
            .get_file(file_id, user_id)
            .await?
            .ok_or("Temporary file doesn't exist")?;
        tokio::fs::rename(&Path::new(temp_file.file_path.as_str()), new_path).await?;

        self.temp_file_repo.delete_file(file_id).await?;
        Ok(())
    }
}
