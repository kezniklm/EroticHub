use crate::persistence::entities::user::User;
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

#[async_trait]
pub trait UserRepoTrait: Debug {
    async fn create_user(&self, user: User) -> anyhow::Result<User>;
    async fn get_user_by_id(&self, user_id: i32) -> anyhow::Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<User>>;
    async fn get_user_by_id_full(&self, user_id: i32) -> anyhow::Result<Option<User>>;
    async fn get_user_by_username_full(&self, username: &str) -> anyhow::Result<Option<User>>;
    async fn update_user(&self, user: User) -> anyhow::Result<Option<User>>;
    async fn delete_user(&self, user_id: i32) -> anyhow::Result<bool>;
    async fn list_users(&self) -> anyhow::Result<Vec<User>>;
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
impl UserRepoTrait for UserRepository {
    async fn create_user(&self, user: User) -> anyhow::Result<User> {
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

    async fn get_user_by_id(&self, user_id: i32) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, NULL as password_hash, email, profile_picture_path, artist_id, paying_member_id
            FROM user_table
            WHERE id = $1
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, NULL as password_hash, email, profile_picture_path, artist_id, paying_member_id
            FROM user_table
            WHERE username = $1
            "#,
            username
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn get_user_by_id_full(&self, user_id: i32) -> anyhow::Result<Option<User>> {
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

    async fn get_user_by_username_full(&self, username: &str) -> anyhow::Result<Option<User>> {
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

    async fn delete_user(&self, user_id: i32) -> anyhow::Result<bool> {
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

    async fn list_users(&self) -> anyhow::Result<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id
            FROM user_table
            "#
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }
}
