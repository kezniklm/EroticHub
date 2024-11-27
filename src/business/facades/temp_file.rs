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
    async fn persist_temp_file(&self, temp_file: NamedTempFile, user_id: i32) -> anyhow::Result<i32>;
    /// Checks if temp file belongs to current user. Is so, stores it to given path
    async fn get_file_path(&self, user_id: i32, permanent_path: &Path);
    fn get_temp_directory(&self) -> String;
    async fn create_temp_directory(&self) -> anyhow::Result<()>;
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
    // TODO: Clear database table and directory before each application start!
}

#[async_trait]
impl TempFileFacadeTrait for TempFileFacade {
    async fn persist_temp_file(&self, temp_file: NamedTempFile, user_id: i32) -> anyhow::Result<i32> {
        let uuid = Uuid::new_v4();
        let path_str = format!("./{}/{}.mp4", self.get_temp_directory(), uuid);
        tokio::fs::copy(temp_file.path(), &path_str).await?;
        temp_file.close()?;

        let entity = TempFile {
            id: -1,
            user_id,
            file_path: path_str.clone(),
        };
        let temp_file_id = self.temp_file_repo.add_file(entity).await?;

        debug!("Stored temp file with ID: {} and path: {}", &temp_file_id, &path_str);
        Ok(temp_file_id)
    }

    async fn get_file_path(&self, user_id: i32, permanent_path: &Path) {
        todo!()
    }

    fn get_temp_directory(&self) -> String {
        dotenvy::var(TEMP_DIRECTORY_KEY).unwrap_or(DEFAULT_TEMP_DIRECTORY.to_string())
    }

    async fn create_temp_directory(&self) -> anyhow::Result<()> {
        let temp_directory = self.get_temp_directory();
        let path = Path::new(temp_directory.as_str());
        if path.exists() {
            return Ok(());
        }
        create_dir(path).await?;
        Ok(())
    }
}