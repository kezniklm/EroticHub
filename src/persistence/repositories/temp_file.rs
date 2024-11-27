use crate::persistence::entities::temp_file::TempFile;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

#[async_trait]
pub trait TempFileRepo {
    async fn add_file(&self, temp_file: TempFile) -> anyhow::Result<i32>;
    async fn get_file(&self, file_id: i32, user_id: i32) -> anyhow::Result<Option<TempFile>>;
}

pub struct PgTempFileRepo {
    pg_pool: Arc<PgPool>,
}

#[async_trait]
impl TempFileRepo for PgTempFileRepo {
    async fn add_file(&self, temp_file: TempFile) -> anyhow::Result<i32> {
        let result = sqlx::query!(
            r#"INSERT INTO 
            temp_file (user_id, file_path) 
            VALUES ($1, $2)
            RETURNING id"#,
            temp_file.user_id,
            temp_file.file_path
        )
        .fetch_one(self.pg_pool.clone().as_ref())
        .await?;
        
        Ok(result.id)
    }

    async fn get_file(&self, file_id: i32, user_id: i32) -> anyhow::Result<Option<TempFile>> {
        let result = sqlx::query_as!(TempFile, r#"
            SELECT f.id, f.user_id, f.file_path FROM temp_file f JOIN user_table u ON f.id = u.id
            WHERE f.id=$1 AND u.id=$2;
        "#, file_id, user_id).fetch_optional(self.pg_pool.clone().as_ref()).await?;
        
        Ok(result)
    }
}
