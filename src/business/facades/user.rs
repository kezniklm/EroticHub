use crate::business::mappers::generic::ToMappedList;
use crate::business::models::user_detail::UserDetail;
use crate::business::models::user_list::UserList;
use crate::persistence::repositories::user::UserRepositoryTrait;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait UserFacadeTrait {
    async fn list_users(&self) -> anyhow::Result<Vec<UserDetail>>;
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
}
