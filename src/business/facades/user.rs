use crate::api::permissions::roles::UserRole;
use crate::business::mappers::generic::ToMappedList;
use crate::business::models::user_detail::UserDetail;
use crate::business::models::user_list::UserList;
use crate::business::models::user_register::UserRegister;
use crate::business::validation::contexts::user::UserValidationContext;
use crate::business::validation::validatable::Validatable;
use crate::persistence::entities::user::User;
use crate::persistence::repositories::user::UserRepositoryTrait;
use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait UserFacadeTrait {
    async fn list_users(&self) -> anyhow::Result<Vec<UserDetail>>;
    async fn register(&self, user_register_model: UserRegister) -> anyhow::Result<()>;
    async fn get_permissions(&self, user_id: i32) -> anyhow::Result<HashSet<String>>;
}

#[derive(Debug, Clone)]
pub struct UserFacade {
    user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,
}

impl UserFacade {
    pub fn new(user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl UserFacadeTrait for UserFacade {
    async fn list_users(&self) -> anyhow::Result<Vec<UserDetail>> {
        let users = self.user_repository.list_users().await?;

        let users2 = users.clone(); //TODO REMOVE - used only as an example

        let user_details = users.to_mapped_list(UserDetail::from);

        let _user_lists_example = users2.to_mapped_list(UserList::from); //TODO REMOVE - used only as an example

        Ok(user_details)
    }

    async fn register(&self, user_register_model: UserRegister) -> anyhow::Result<()> {
        user_register_model
            .validate_model(&UserValidationContext {
                user_repository: Arc::clone(&self.user_repository),
            })
            .await?;

        let password_hash = hash(user_register_model.password.as_str(), DEFAULT_COST)?;

        let mut new_user = User::from(user_register_model);

        new_user.password_hash = Some(password_hash);

        self.user_repository.create_user(new_user).await?;

        Ok(())
    }

    async fn get_permissions(&self, user_id: i32) -> anyhow::Result<HashSet<String>> {
        let user = match self.user_repository.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Ok(HashSet::new()),
        };

        let mut user_permissions = HashSet::new();

        user_permissions.insert(UserRole::Registered);

        if user.paying_member_id.is_some() {
            user_permissions.insert(UserRole::PayingMember);
        }

        if user.artist_id.is_some() {
            user_permissions.insert(UserRole::Artist);
        }

        Ok(user_permissions
            .iter()
            .map(|role| role.to_string())
            .collect::<HashSet<String>>())
    }
}
