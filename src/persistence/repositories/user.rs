use crate::persistence::entities::user::User;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait UserRepositoryTrait: Debug {
    async fn create_user(&self, user: User) -> Result<User>;
    async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn update_user(&self, user: User) -> Result<Option<User>>;
    async fn delete_user(&self, user_id: i32) -> Result<bool>;
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
            RETURNING id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id
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
            SELECT id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id
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
            SELECT id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id
            FROM user_table
            WHERE username = $1
            "#,
            username
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn update_user(&self, user: User) -> anyhow::Result<Option<User>> {
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
                paying_member_id = $6
            WHERE id = $7
            RETURNING id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id
            "#,
            user.username,
            user.password_hash,
            user.email,
            user.profile_picture_path,
            user.artist_id,
            user.paying_member_id,
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
}

#[cfg(test)]
mod tests {
    use crate::common::tests::setup::AsyncContext;
    use crate::persistence::entities::user::User;
    use crate::persistence::repositories::user::{UserRepository, UserRepositoryTrait};
    use crate::persistence::Result;
    use test_context::test_context;

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn test_create_user(context: &AsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "test_user".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "test_user@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
        };

        let created_user = user_repo.create_user(new_user.clone()).await?;

        assert_eq!(created_user.username, new_user.username);
        assert_eq!(created_user.email, new_user.email);
        assert!(created_user.id > 0);

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn test_get_user_by_id(context: &AsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "get_user_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "get_user_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
        };

        let created_user = user_repo.create_user(new_user).await?;

        let fetched_user = user_repo.get_user_by_id(created_user.id).await?;
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().id, created_user.id);

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn test_get_user_by_username(context: &AsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "get_user_by_username_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "get_user_by_username_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
        };

        let created_user = user_repo.create_user(new_user).await?;

        let fetched_user = user_repo
            .get_user_by_username(&created_user.username)
            .await?;
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().username, created_user.username);

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn test_update_user(context: &AsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "update_user_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "update_user_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
        };

        let created_user = user_repo.create_user(new_user.clone()).await?;

        let mut user_to_update = created_user;
        user_to_update.email = "updated_email@example.com".to_string();

        let updated_user = user_repo.update_user(user_to_update.clone()).await?;
        assert!(updated_user.is_some());
        assert_eq!(updated_user.unwrap().email, "updated_email@example.com");

        Ok(())
    }

    #[test_context(AsyncContext)]
    #[tokio::test]
    async fn test_delete_user(context: &AsyncContext) -> Result<()> {
        let user_repo = UserRepository::new(context.pg_pool.clone());

        let new_user = User {
            id: 0, // id will be auto-generated
            username: "delete_user_test".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email: "delete_user_test@example.com".to_string(),
            profile_picture_path: Some("path/to/pic.jpg".to_string()),
            artist_id: None,
            paying_member_id: None,
        };
        let created_user = user_repo.create_user(new_user).await?;

        let is_deleted = user_repo.delete_user(created_user.id).await?;
        assert!(is_deleted);

        let fetched_user = user_repo.get_user_by_id(created_user.id).await?;
        assert!(fetched_user.is_none());

        Ok(())
    }
}
