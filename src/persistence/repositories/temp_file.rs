use std::path::Path;
use crate::persistence::entities::temp_file::TempFile;
use async_trait::async_trait;
use sqlx::PgPool;
use tempfile::NamedTempFile;

#[async_trait]
pub trait TempFileRepo {
    async fn add_file(&self, temp_file_entity: TempFile, temp_file: NamedTempFile) -> anyhow::Result<i32>;
    async fn get_file(&self, file_id: i32, user_id: i32) -> anyhow::Result<Option<TempFile>>;
    async fn delete_all_files(&self, temp_directory_path: &Path) -> anyhow::Result<()>;
}

pub struct PgTempFileRepo {
    pg_pool: PgPool,
}

impl PgTempFileRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool,
        }
    }
}

#[async_trait]
impl TempFileRepo for PgTempFileRepo {
    async fn add_file(&self, temp_file_entity: TempFile, temp_file: NamedTempFile) -> anyhow::Result<i32> {
        let mut transaction = self.pg_pool.begin().await?;

        tokio::fs::copy(temp_file.path(), &temp_file_entity.file_path).await?;
        temp_file.close()?;

        let result = sqlx::query!(
            r#"INSERT INTO 
            temp_file (user_id, file_path) 
            VALUES ($1, $2)
            RETURNING id"#,
            temp_file_entity.user_id,
            temp_file_entity.file_path
        )
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;
        
        Ok(result.id)
    }

    async fn get_file(&self, file_id: i32, user_id: i32) -> anyhow::Result<Option<TempFile>> {
        let result = sqlx::query_as!(TempFile, r#"
            SELECT f.id, f.user_id, f.file_path FROM temp_file f JOIN user_table u ON f.id = u.id
            WHERE f.id=$1 AND u.id=$2;
        "#, file_id, user_id).fetch_optional(&self.pg_pool).await?;
        
        Ok(result)
    }

    async fn delete_all_files(&self, temp_directory_path: &Path) -> anyhow::Result<()> {
        let mut transaction = self.pg_pool.begin().await?;
        
        sqlx::query!("DELETE FROM temp_file").execute(&mut *transaction).await?;
        tokio::fs::remove_dir_all(temp_directory_path).await?;
        tokio::fs::create_dir(temp_directory_path).await?;
        
        transaction.commit().await?;
        Ok(())
    }
}
