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
        .await?;

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
        sqlx::query!(
            "DELETE FROM temp_file WHERE id=$1 AND user_id=$2 RETURNING *",
            file_id,
            user_id
        )
        .fetch_one(&self.pg_pool)
        .await
        .db_error("Failed to delete temporary file")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::tests::setup::EmptyAsyncContext;
    use test_context::test_context;

    static TEMP_FILE_NAME: &str = "temp_file";

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn add_temp_file(ctx: &mut EmptyAsyncContext) {
        create_dummy_user(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy user");

        let repo = create_repo(ctx.pg_pool.clone());
        let path = get_temp_file_path(ctx);

        let (temp_file, entity) = create_entity(None, &path);

        let file = repo
            .add_file(entity, temp_file)
            .await
            .expect("Failed to save temporary file");
        assert_eq!(file, 1, "ID of new temporary file doesn't match");
        tokio::fs::read(path)
            .await
            .expect("Temporary file should exist");
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn get_for_user(ctx: &mut EmptyAsyncContext) {
        create_dummy_user(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy user");

        let repo = create_repo(ctx.pg_pool.clone());

        let (temp_file, entity) = create_entity(Some(1), &get_temp_file_path(ctx));
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

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn delete_temp_file(ctx: &mut EmptyAsyncContext) -> Result<()> {
        create_dummy_user(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy user");
        tokio::fs::create_dir_all("./test_resources")
            .await
            .expect("Failed to create test directory"); // TODO: temporary

        let repo = create_repo(ctx.pg_pool.clone());
        let (temp_file, entity) = create_entity(Some(1), &get_temp_file_path(ctx));
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

        Ok(())
    }

    fn create_entity(id: Option<i32>, temp_file_path: &str) -> (NamedTempFile, TempFile) {
        let temp_file = NamedTempFile::new().expect("Failed to created temporary file");

        let temp_file_entity = TempFile {
            id: id.unwrap_or(-1),
            user_id: 1,
            file_path: temp_file_path.to_string(),
        };

        (temp_file, temp_file_entity)
    }

    fn create_repo(pg_pool: PgPool) -> impl TempFileRepo {
        PgTempFileRepo::new(pg_pool)
    }

    fn get_temp_file_path(ctx: &EmptyAsyncContext) -> String {
        format!("{}/{}", ctx.test_folders_root, TEMP_FILE_NAME)
    }

    async fn create_dummy_user(pg_pool: &PgPool) -> Result<()> {
        sqlx::query!(r#"INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id) 
                    VALUES (1, 'John', 'hash', 'email@email.cz', 'path/pic.png', null, null);"#)
            .execute(pg_pool).await?;

        Ok(())
    }
}
