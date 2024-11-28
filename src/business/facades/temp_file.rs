use std::path::Path;
use std::sync::Arc;
use async_trait::async_trait;
use log::debug;
use tempfile::NamedTempFile;
use tokio::fs::create_dir;
use uuid::Uuid;
use crate::persistence::entities::temp_file::TempFile;
use crate::persistence::repositories::temp_file::TempFileRepo;

const DEFAULT_TEMP_DIRECTORY: &str = "temp";
const TEMP_DIRECTORY_KEY: &str = "TEMP_DIRECTORY_PATH";
#[async_trait]
pub trait TempFileFacadeTrait {
    async fn persist_temp_file(&self, temp_file: NamedTempFile, file_name: String, user_id: i32) -> anyhow::Result<i32>;
    /// Checks if temp file belongs to current user. Is so, stores it to given path
    async fn get_file_path(&self, user_id: i32, permanent_path: &Path);
    fn get_temp_directory_path(&self) -> String;
    async fn create_temp_directory(&self) -> anyhow::Result<()>;
    async fn delete_all_temp_files(&self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct TempFileFacade {
    temp_file_repo: Arc<dyn TempFileRepo + Sync + Send>,
}

impl TempFileFacade {
    pub fn new(temp_file_repo: Arc<dyn TempFileRepo + Sync + Send>) -> Self {
        Self {
            temp_file_repo,
        }
    }

    fn get_file_extension(&self, file_name: String) -> String {
        if let Some(file_name) = file_name.split_once(".") {
            let (_name, extension) = file_name;
            return extension.to_string();
        }
        String::new()
    }
    // TODO: Clear database table and directory before each application start!
}

#[async_trait]
impl TempFileFacadeTrait for TempFileFacade {
    async fn persist_temp_file(&self, temp_file: NamedTempFile, file_name: String, user_id: i32) -> anyhow::Result<i32> {
        let uuid = Uuid::new_v4();

        let path_str = format!("./{}/{}.{}", self.get_temp_directory_path(), uuid, self.get_file_extension(file_name));
        let entity = TempFile {
            id: -1,
            user_id,
            file_path: path_str.clone(),
        };

        let temp_file_id = self.temp_file_repo.add_file(entity, temp_file).await?;

        debug!("Stored temp file with ID: {} and path: {}", &temp_file_id, &path_str);
        Ok(temp_file_id)
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
            return Ok(())
        }
        self.temp_file_repo.delete_all_files(temp_dir_path).await?;
        
        debug!("All temp files were deleted!");
        Ok(())
    }
}