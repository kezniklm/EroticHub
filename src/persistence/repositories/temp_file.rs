use crate::persistence::entities::error::{DatabaseError, MapToDatabaseError};
use crate::persistence::entities::temp_file::TempFile;
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use std::path::Path;
use tempfile::NamedTempFile;

#[async_trait]
pub trait TempFileRepo {
    async fn add_file(
        &self,
        temp_file_entity: TempFile,
        temp_file: NamedTempFile,
    ) -> anyhow::Result<i32, DatabaseError>;
    async fn get_file(&self, file_id: i32, user_id: i32) -> Result<TempFile>;
    async fn delete_all_files(&self, temp_directory_path: &Path) -> Result<()>;
    /// Deletes temporary file from the file system and database
    ///
    /// # Returns
    /// `bool` if deletion was successful
    async fn delete_file(&self, file_id: i32, user_id: i32) -> Result<()>;
}

pub struct PgTempFileRepo {
    pg_pool: PgPool,
}

impl PgTempFileRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl TempFileRepo for PgTempFileRepo {
    async fn add_file(
        &self,
        temp_file_entity: TempFile,
        temp_file: NamedTempFile,
    ) -> Result<i32, DatabaseError> {
        let mut transaction = self.pg_pool.begin().await?;

        tokio::fs::copy(temp_file.path(), &temp_file_entity.file_path)
            .await
            .db_error("Failed to copy the temp file")?;
        temp_file
            .close()
            .db_error("Failed to close temporary file")?;

        let result = sqlx::query!(
            r#"INSERT INTO 
            temp_file (user_id, file_path) 
            VALUES ($1, $2)
            RETURNING id"#,
            temp_file_entity.user_id,
            temp_file_entity.file_path
        )
        .fetch_one(&mut *transaction)
        .await
        .db_error("Failed to create temporary file")?;

        transaction.commit().await?;

        Ok(result.id)
    }

    async fn get_file(&self, file_id: i32, user_id: i32) -> Result<TempFile> {
        let result = sqlx::query_as!(
            TempFile,
            r#"
            SELECT f.id, f.user_id, f.file_path FROM temp_file f
            WHERE f.id=$1 AND f.user_id=$2;
        "#,
            file_id,
            user_id
        )
        .fetch_one(&self.pg_pool)
        .await
        .db_error("Temporary file doesn't exist")?;

        Ok(result)
    }

    async fn delete_all_files(&self, temp_directory_path: &Path) -> Result<()> {
        let mut transaction = self.pg_pool.begin().await?;

        sqlx::query!("DELETE FROM temp_file")
            .execute(&mut *transaction)
            .await?;
        tokio::fs::remove_dir_all(temp_directory_path)
            .await
            .db_error("Failed to delete temporary files!")?;
        tokio::fs::create_dir_all(temp_directory_path)
            .await
            .db_error("Failed to delete temporary files!")?;

        transaction.commit().await?;
        Ok(())
    }

    async fn delete_file(&self, file_id: i32, user_id: i32) -> Result<()> {
        let mut transcation = self.pg_pool.begin().await?;
        let deleted_file = sqlx::query_as!(
            TempFile,
            "DELETE FROM temp_file WHERE id=$1 AND user_id=$2 RETURNING *",
            file_id,
            user_id
        )
        .fetch_one(&mut *transcation)
        .await
        .db_error("Failed to delete temporary file")?;

        tokio::fs::remove_file(deleted_file.file_path)
            .await
            .db_error("Failed to delete temporary file")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::tests::setup::AsyncContext;
    use test_context::test_context;

    static TEMP_FILE_PATH: &str = "./test_resources/temp_file"; // TODO: temporary

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn add_temp_file(ctx: &mut AsyncContext) {
        create_dummy_user(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy user");
        tokio::fs::create_dir_all("./test_resources")
            .await
            .expect("Failed to create test directory"); // TODO: temporary

        let repo = create_repo(ctx.pg_pool.clone());

        let (temp_file, entity) = create_entity(None);

        let file = repo
            .add_file(entity, temp_file)
            .await
            .expect("Failed to save temporary file");
        assert_eq!(file, 1, "ID of new temporary file doesn't match");
        tokio::fs::read(TEMP_FILE_PATH)
            .await
            .expect("Temporary file should exist");

        tokio::fs::remove_file(TEMP_FILE_PATH).await.unwrap();
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn get_for_user(ctx: &mut AsyncContext) {
        create_dummy_user(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy user");
        tokio::fs::create_dir_all("./test_resources")
            .await
            .expect("Failed to create test directory"); // TODO: temporary

        let repo = create_repo(ctx.pg_pool.clone());

        let (temp_file, entity) = create_entity(Some(1));
        let file = repo
            .add_file(entity.clone(), temp_file)
            .await
            .expect("Failed to save temporary file");

        let fetched_entity = repo
            .get_file(file, 1)
            .await
            .expect("Failed to fetch existing temporary file");
        assert_eq!(entity, fetched_entity, "Entities doesn't match");

        let fetched_entity = repo.get_file(file, 2).await;
        let expected_result: Result<TempFile> =
            Err(DatabaseError::new("Temporary file doesn't exist"));

        assert_eq!(fetched_entity, expected_result);
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn delete_temp_file(ctx: &mut AsyncContext) -> Result<()> {
        create_dummy_user(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy user");
        tokio::fs::create_dir_all("./test_resources")
            .await
            .expect("Failed to create test directory"); // TODO: temporary

        let repo = create_repo(ctx.pg_pool.clone());
        let (temp_file, entity) = create_entity(Some(1));
        let file = repo
            .add_file(entity.clone(), temp_file)
            .await
            .expect("Failed to save temporary file");

        repo.delete_file(file, 1)
            .await
            .expect("Failed to delete temporary file");
        let delete_result = repo.delete_file(file, 2).await;
        let expected_result: Result<()> =
            Err(DatabaseError::new("Failed to delete temporary file"));

        assert_eq!(
            delete_result, expected_result,
            "It's possible to delete temp file with user which didn't created it"
        );

        let file_exists = tokio::fs::try_exists(entity.file_path).await?;
        assert!(
            !file_exists,
            "Deleted temporary file still exists on file system"
        );

        Ok(())
    }

    fn create_entity(id: Option<i32>) -> (NamedTempFile, TempFile) {
        let temp_file = NamedTempFile::new().expect("Failed to created temporary file");

        let temp_file_entity = TempFile {
            id: id.unwrap_or(-1),
            user_id: 1,
            file_path: String::from(TEMP_FILE_PATH),
        };

        (temp_file, temp_file_entity)
    }

    fn create_repo(pg_pool: PgPool) -> impl TempFileRepo {
        PgTempFileRepo::new(pg_pool)
    }

    async fn create_dummy_user(pg_pool: &PgPool) -> Result<()> {
        sqlx::query!(r#"INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id) 
                    VALUES (1, 'John', 'hash', 'email@email.cz', 'path/pic.png', null, null);"#)
            .execute(pg_pool).await?;

        Ok(())
    }
}
