use crate::persistence::entities::user::{LikedVideo, User, UserName};
use crate::persistence::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait UserRepositoryTrait: Debug {
    async fn create_user(&self, user: User) -> Result<User>;
    async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn update_user(&self, user: User) -> Result<Option<User>>;
    async fn delete_user(&self, user_id: i32) -> Result<bool>;
    async fn fetch_usernames_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<UserName>>;
    async fn get_users(&self) -> Result<Vec<User>>;
    async fn change_admin_status(&self, user_id: i32, is_admin: bool) -> Result<()>;
    async fn is_liked_already(&self, user_id: i32, video_id: i32) -> Result<bool>;
    async fn liked_videos_by_user(&self, user_id: i32) -> Result<Vec<LikedVideo>>;
    async fn like_video(&self, user_id: i32, video_id: i32) -> Result<()>;
    async fn unlike_video(&self, user_id: i32, video_id: i32) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn create_user(&self, user: User) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO user_table (username, password_hash, email, profile_picture_path, artist_id, paying_member_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id, is_admin
            "#,
            user.username,
            user.password_hash,
            user.email,
            user.profile_picture_path,
            user.artist_id,
            user.paying_member_id
        )
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id, is_admin
            FROM user_table
            WHERE id = $1
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id, is_admin
            FROM user_table
            WHERE username = $1
            "#,
            username
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id, is_admin
            FROM user_table
            WHERE email = $1
            "#,
            email
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn fetch_usernames_by_id(&self, ids: Vec<i32>) -> anyhow::Result<Vec<UserName>> {
        let users = sqlx::query_as!(
            UserName,
            r#"
            SELECT id, username
            FROM user_table
            WHERE id IN (SELECT unnest($1::integer[]))
            "#,
            &ids
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    async fn update_user(&self, user: User) -> Result<Option<User>> {
        let updated_user = sqlx::query_as!(
            User,
            r#"
            UPDATE user_table
            SET
                username = $1,
                password_hash = $2,
                email = $3,
                profile_picture_path = $4,
                artist_id = $5,
                paying_member_id = $6,
                is_admin = $7
            WHERE id = $8
            RETURNING id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id, is_admin
            "#,
            user.username,
            user.password_hash,
            user.email,
            user.profile_picture_path,
            user.artist_id,
            user.paying_member_id,
            user.is_admin,
            user.id
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(updated_user)
    }

    async fn delete_user(&self, user_id: i32) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM user_table
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn get_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id, is_admin
            FROM user_table
            ORDER BY username ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    async fn change_admin_status(&self, user_id: i32, is_admin: bool) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE user_table
            SET is_admin = $1
            WHERE id = $2
            "#,
            is_admin,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn is_liked_already(&self, user_id: i32, video_id: i32) -> Result<bool> {
        let like = sqlx::query!(
            r#"
            SELECT user_id, video_id
            FROM favorite
            WHERE user_id = $1 and video_id = $2
            "#,
            user_id,
            video_id
        )
        .fetch_optional(&self.pool) // Use fetch_optional to avoid an error if no row is found
        .await?;
        Ok(like.is_some())
    }

    async fn liked_videos_by_user(&self, user_id: i32) -> Result<Vec<LikedVideo>> {
        let likes = sqlx::query_as!(
            LikedVideo,
            r#"
            SELECT user_id, video_id
            FROM favorite
            WHERE user_id = $1
            "#,
            user_id,
        )
        .fetch_all(&self.pool) // Use fetch_optional to avoid an error if no row is found
        .await?;
        Ok(likes)
    }

    async fn like_video(&self, user_id: i32, video_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO favorite (user_id, video_id)
            VALUES ($1, $2)
            RETURNING user_id, video_id
            "#,
            user_id,
            video_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(())
    }

    async fn unlike_video(&self, user_id: i32, video_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM favorite
            WHERE user_id = $1 and video_id = $2
            "#,
            user_id,
            video_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::tests::setup::EmptyAsyncContext;
    use crate::persistence::entities::user::User;
    use crate::persistence::repositories::user::{UserRepository, UserRepositoryTrait};
    use crate::persistence::Result;
    use test_context::test_context;

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_create_user(context: &EmptyAsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "test_user".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "test_user@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
            is_admin: false,
        };

        let created_user = user_repo.create_user(new_user.clone()).await?;

        assert_eq!(created_user.username, new_user.username);
        assert_eq!(created_user.email, new_user.email);
        assert!(created_user.id > 0);

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_get_user_by_id(context: &EmptyAsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "get_user_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "get_user_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
            is_admin: false,
        };

        let created_user = user_repo.create_user(new_user).await?;

        let fetched_user = user_repo.get_user_by_id(created_user.id).await?;
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().id, created_user.id);

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_get_user_by_username(context: &EmptyAsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "get_user_by_username_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "get_user_by_username_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
            is_admin: false,
        };

        let created_user = user_repo.create_user(new_user).await?;

        let fetched_user = user_repo
            .get_user_by_username(&created_user.username)
            .await?;
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().username, created_user.username);

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_get_user_by_email(context: &EmptyAsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "get_user_by_email_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "get_user_by_email_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
            is_admin: false,
        };

        let created_user = user_repo.create_user(new_user).await?;

        let fetched_user = user_repo.get_user_by_email(&created_user.email).await?;
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().username, created_user.username);

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_update_user(context: &EmptyAsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "update_user_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "update_user_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
            is_admin: false,
        };

        let created_user = user_repo.create_user(new_user.clone()).await?;

        let mut user_to_update = created_user;
        user_to_update.email = "updated_email@example.com".to_string();

        let updated_user = user_repo.update_user(user_to_update.clone()).await?;
        assert!(updated_user.is_some());
        assert_eq!(updated_user.unwrap().email, "updated_email@example.com");

        Ok(())
    }

    #[test_context(EmptyAsyncContext)]
    #[tokio::test]
    async fn test_delete_user(context: &EmptyAsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "delete_user_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "delete_user_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
            is_admin: false,
        };
        let created_user = user_repo.create_user(new_user).await?;

        let is_deleted = user_repo.delete_user(created_user.id).await?;
        assert!(is_deleted);

        let fetched_user = user_repo.get_user_by_id(created_user.id).await?;
        assert!(fetched_user.is_none());

        Ok(())
    }
}
