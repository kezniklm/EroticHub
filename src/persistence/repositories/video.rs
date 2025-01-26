use crate::persistence::entities::error::MapToDatabaseError;
use crate::persistence::entities::video::{PatchVideo, Video, VideoVisibility};
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};

#[async_trait]
pub trait VideoRepo {
    #[allow(dead_code)]
    async fn list_videos(&self) -> anyhow::Result<Vec<Video>>;
    async fn save_video(&self, video: Video, tx: &mut Transaction<Postgres>) -> Result<Video>;
    async fn patch_video(&self, video: PatchVideo, tx: &mut Transaction<Postgres>)
        -> Result<Video>;
    async fn delete_video(
        &self,
        video_id: i32,
        artist_id: i32,
        tx: &mut Transaction<Postgres>,
    ) -> Result<Option<Video>>;
    async fn get_video_by_id(
        &self,
        video_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<Video>>;
    async fn get_video_artist_id(
        &self,
        video_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<i32>;
    async fn fetch_videos(
        &self,
        ord: Option<&str>,
        filter: Option<Vec<i32>>,
        offset: Option<i32>,
    ) -> anyhow::Result<Vec<Video>>;
}

#[derive(Debug, Clone)]
pub struct PgVideoRepo {
    pg_pool: PgPool,
}

impl PgVideoRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }

    async fn remove_old_file<'a>(file_path: &str) -> Result<()> {
        tokio::fs::remove_file(file_path)
            .await
            .db_error("Failed to remove old file")?;
        Ok(())
    }
}

#[async_trait]
impl VideoRepo for PgVideoRepo {
    async fn list_videos(&self) -> anyhow::Result<Vec<Video>> {
        let result = sqlx::query_as!(
            Video,
            r#"SELECT
            id,
            artist_id,
            visibility AS "visibility: VideoVisibility",
            name,
            file_path,
            thumbnail_path,
            description FROM video ORDER BY id"#
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(result)
    }

    async fn save_video(&self, video: Video, tx: &mut Transaction<Postgres>) -> Result<Video> {
        let result = sqlx::query_as!(
            Video,
            r#"
            INSERT INTO video(
                artist_id,
                name,
                file_path,
                thumbnail_path,
                description,
                visibility
            ) VALUES ($1, $2, $3, $4, $5, $6) 
            RETURNING id, artist_id, visibility AS "visibility: VideoVisibility",
            name, file_path, thumbnail_path, description 
        "#,
            video.artist_id,
            video.name,
            video.file_path,
            video.thumbnail_path,
            video.description,
            video.visibility as VideoVisibility
        )
        .fetch_one(tx.as_mut())
        .await?;

        Ok(result)
    }

    async fn patch_video(
        &self,
        new_video: PatchVideo,
        tx: &mut Transaction<Postgres>,
    ) -> Result<Video> {
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"UPDATE video SET "#);

        let mut first = true;
        let mut old_video: Option<Video> = None;

        if let Some(artist_id) = new_video.artist_id {
            if !first {
                query.push(",");
            };
            query.push(" artist_id = ");
            query.push_bind(artist_id);

            first = false;
        }

        if let Some(name) = new_video.name {
            if !first {
                query.push(",");
            };

            query.push(" name = ");
            query.push_bind(name);

            first = false;
        }

        if let Some(ref file_path) = new_video.file_path {
            if !first {
                query.push(",");
            };

            query.push(" file_path = ");
            query.push_bind(file_path);

            first = false;
            old_video = match old_video {
                None => self.get_video_by_id(new_video.id, None).await?,
                Some(video) => Some(video),
            }
        }

        if let Some(ref thumbnail_path) = new_video.thumbnail_path {
            if !first {
                query.push(",");
            };

            query.push(" thumbnail_path = ");
            query.push_bind(thumbnail_path);

            first = false;

            old_video = match old_video {
                None => self.get_video_by_id(new_video.id, None).await?,
                Some(video) => Some(video),
            }
        }

        if let Some(description) = new_video.description {
            if !first {
                query.push(",");
            };

            query.push(" description = ");
            query.push_bind(description);
        }

        query.push(", visibility = ");
        query.push_bind(new_video.visibility);

        query.push(" WHERE id = ");
        query.push_bind(new_video.id);
        query.push(" RETURNING *");

        let result: Video = query.build_query_as().fetch_one(tx.as_mut()).await?;

        if let Some(old_video) = old_video {
            if new_video.file_path.is_some() {
                Self::remove_old_file(old_video.file_path.as_str()).await?;
            }

            if new_video.thumbnail_path.is_some() {
                Self::remove_old_file(old_video.thumbnail_path.as_str()).await?;
            }
        }

        Ok(result)
    }

    async fn delete_video(
        &self,
        video_id: i32,
        user_id: i32,
        tx: &mut Transaction<Postgres>,
    ) -> Result<Option<Video>> {
        let video_optional = sqlx::query_as!(
            Video,
            r#"DELETE FROM video WHERE id = $1 AND artist_id = $2
            RETURNING id, artist_id, visibility AS "visibility: VideoVisibility",
            name, file_path, thumbnail_path, description "#,
            video_id,
            user_id
        )
        .fetch_optional(tx.as_mut())
        .await?;

        match video_optional {
            None => Ok(None),
            Some(video) => {
                Self::remove_old_file(&video.file_path).await?;
                Self::remove_old_file(&video.thumbnail_path).await?;

                Ok(Some(video))
            }
        }
    }

    async fn get_video_by_id(
        &self,
        video_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<Video>> {
        let query = sqlx::query_as!(
            Video,
            r#"
            SELECT 
            id, 
            artist_id, 
            visibility AS "visibility: VideoVisibility", 
            name, 
            file_path, 
            thumbnail_path, 
            description
            FROM video WHERE id = $1
            "#,
            video_id
        );

        let result = match tx {
            None => query.fetch_optional(&self.pg_pool).await,
            Some(tx) => query.fetch_optional(tx.as_mut()).await,
        }
        .db_error("Video doesn't exist")?;

        Ok(result)
    }

    async fn get_video_artist_id(
        &self,
        video_id: i32,
        tx: Option<&mut Transaction<Postgres>>,
    ) -> Result<i32> {
        let query = sqlx::query!("SELECT artist_id FROM video WHERE id = $1", video_id);
        let record = match tx {
            None => query.fetch_one(&self.pg_pool).await,
            Some(tx) => query.fetch_one(tx.as_mut()).await,
        }
        .db_error("Failed to get artist id of the video")?;

        Ok(record.artist_id)
    }

    async fn fetch_videos(
        &self,
        ord: Option<&str>,
        filter: Option<Vec<i32>>,
        offset: Option<i32>,
    ) -> anyhow::Result<Vec<Video>> {
        let mut query = QueryBuilder::new(
            r#"SELECT
            id,
            artist_id,
            name,
            visibility,
            file_path,
            thumbnail_path,
            description FROM video"#,
        );

        // did not really find a better solution(it works), may be you will
        if let Some(filter) = filter {
            query.push(" JOIN video_category_video ON id = video_id ");
            if filter.len() > 1 {
                let mut filter_array = String::new();
                filter_array.push_str(
                    &filter
                        .iter()
                        .map(|&x| x.to_string()) // Convert each i32 to String
                        .collect::<Vec<String>>()
                        .join(","),
                );
                query.push(format!(
                    " WHERE video.id IN (SELECT video_id FROM video_category_video \
                    WHERE category_id IN ({})
                    GROUP BY
                        video_id
                    HAVING
                    COUNT(DISTINCT category_id) = {})",
                    filter_array,
                    filter.len()
                ));
            } else {
                query.push(" WHERE category_id = ");
                query.push_bind(filter[0]);
            }
        }

        query.push(" GROUP BY video.id ");

        if let Some(ord) = ord {
            query.push(" ORDER BY ");
            query.push_bind(ord);
            query.push(" ");
        }

        query.push(" LIMIT 20");
        if let Some(offset) = offset {
            query.push(" OFFSET ");
            query.push_bind(offset);
        }

        let result = query.build_query_as().fetch_all(&self.pg_pool).await;

        Ok(result?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::tests::setup::EmptyAsyncContext;
    use test_context::test_context;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use uuid::Uuid;

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_save_fetch_video(ctx: &mut EmptyAsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());

        let created_video = create_test_video(Some(1), None, &ctx.test_folders_root).await;
        let mut tx = ctx.pg_pool.begin().await?;

        repo.save_video(created_video.clone(), &mut tx)
            .await
            .expect("Failed to save video");

        tx.commit().await?;
        let video = repo
            .get_video_by_id(created_video.id, None)
            .await
            .db_error("Failed to fetch video")?;
        assert_eq!(Some(created_video), video);

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_save_sequence(ctx: &mut EmptyAsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());

        let mut tx = ctx.pg_pool.begin().await?;

        repo.save_video(
            create_test_video(None, None, &ctx.test_folders_root).await,
            &mut tx,
        )
        .await?;
        repo.save_video(
            create_test_video(None, None, &ctx.test_folders_root).await,
            &mut tx,
        )
        .await?;

        tx.commit().await?;
        let video = repo.get_video_by_id(2, None).await?.unwrap();
        assert_eq!(
            video.id, 2,
            "Sequence should be used for ID of created video"
        );

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_list_videos(ctx: &mut EmptyAsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());
        let mut tx = ctx.pg_pool.begin().await?;

        let video1 = repo
            .save_video(
                create_test_video(Some(1), None, &ctx.test_folders_root).await,
                &mut tx,
            )
            .await?;
        let video2 = repo
            .save_video(
                create_test_video(Some(2), None, &ctx.test_folders_root).await,
                &mut tx,
            )
            .await?;

        tx.commit().await?;
        let video = repo
            .list_videos()
            .await
            .db_error("Failed to fetch videos")?;
        assert_eq!(video.len(), 2, "Unexpected number of videos");
        assert_eq!(Some(&video1), video.first());
        assert_eq!(Some(&video2), video.get(1));

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn video_doesnt_exist(ctx: &mut EmptyAsyncContext) -> Result<()> {
        let repo = create_repository(ctx.pg_pool.clone());

        let video = repo.get_video_by_id(2, None).await?;
        assert_eq!(video, None, "Repository returned unexpected result");

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn patch_video(ctx: &mut EmptyAsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());
        let mut tx = ctx.pg_pool.begin().await?;
        let video1 = repo
            .save_video(
                create_test_video(Some(1), None, &ctx.test_folders_root).await,
                &mut tx,
            )
            .await?;
        tx.commit().await?;

        let mut tx = ctx.pg_pool.begin().await?;

        let edited_video = PatchVideo {
            id: video1.id,
            artist_id: None,
            visibility: VideoVisibility::Paying,
            name: Some(String::from("John2")),
            file_path: None,
            thumbnail_path: None,
            description: Some(String::from("Description2")),
        };
        let updated_video = repo.patch_video(edited_video, &mut tx).await?;
        tx.commit().await?;
        assert_eq!(updated_video.visibility, VideoVisibility::Paying);
        assert_eq!(updated_video.name, String::from("John2"));
        assert_eq!(
            updated_video.description,
            Some(String::from("Description2"))
        );

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn delete_video(ctx: &mut EmptyAsyncContext) -> Result<()> {
        create_dummy_artist(&ctx.pg_pool)
            .await
            .expect("Failed to create dummy artist");
        let repo = create_repository(ctx.pg_pool.clone());
        let mut tx = ctx.pg_pool.begin().await?;
        let video1 = repo
            .save_video(
                create_test_video(Some(1), None, &ctx.test_folders_root).await,
                &mut tx,
            )
            .await?;
        tx.commit().await?;

        let mut tx = ctx.pg_pool.begin().await?;
        let deleted = repo.delete_video(video1.id, 1, &mut tx).await?;

        assert!(deleted.is_some(), "Video wasn't deleted");

        tx.commit().await?;

        let video = repo.get_video_by_id(video1.id, None).await?;

        assert!(video.is_none(), "Video was not deleted");

        Ok(())
    }

    fn create_repository(pg_pool: PgPool) -> impl VideoRepo {
        PgVideoRepo { pg_pool }
    }

    async fn create_test_video(
        video_id: Option<i32>,
        video_visibility: Option<VideoVisibility>,
        folders_root: &str,
    ) -> Video {
        let file_name = Uuid::new_v4();
        let video_path = format!("{folders_root}/videos/{file_name}");
        let mut file = File::create(&video_path).await.unwrap();
        file.write_all(b"test-content").await.unwrap();

        let thumbnail_path = format!("{folders_root}/thumbnails/{file_name}");
        let mut file = File::create(&thumbnail_path).await.unwrap();
        file.write_all(b"test-content").await.unwrap();

        Video {
            id: video_id.unwrap_or(-1),
            artist_id: 1,
            visibility: video_visibility.unwrap_or(VideoVisibility::All),
            name: String::from("John"),
            file_path: video_path,
            thumbnail_path,
            description: Some(String::from("Description")),
        }
    }

    async fn create_dummy_artist(pg_pool: &PgPool) -> Result<()> {
        sqlx::query!(r#"INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id) VALUES (1, 'John', 'hash', 'email@email.cz', 'path/pic.png', null, null);"#)
            .execute(pg_pool).await?;
        sqlx::query!(
            r#"INSERT INTO artist(id, user_id, description) VALUES (1, 1, 'description')"#
        )
        .execute(pg_pool)
        .await?;
        sqlx::query!(r#"UPDATE artist SET user_id = 1 WHERE user_id = 1"#)
            .execute(pg_pool)
            .await?;

        Ok(())
    }
}
