use crate::business::mappers::generic::ToMappedList;
use crate::business::models::error::AppErrorKind::BadRequestError;
use crate::business::models::error::{AppError, AppErrorKind, MapToAppError};
use crate::business::models::user::{
    ProfilePictureUpdate, UserDetail, UserDetailUpdate, UserLogin, UserPasswordUpdate,
    UserRegister, UserRegisterMultipart, UserRole, Username,
};
use crate::business::util::file::{create_dir_if_not_exist, get_file_extension};
use crate::business::validation::contexts::user::UserValidationContext;
use crate::business::validation::validatable::Validatable;
use crate::business::Result;
use crate::persistence::entities::user::User;
use crate::persistence::repositories::user::UserRepositoryTrait;
use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use log::info;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;
use validator::{Validate, ValidationError};

const PROFILE_PICTURE_FOLDER_PATH: &str = "resources/images/users/";
const VALIDATION_ERROR_TEXT: &str = "Validation failed";

const ALLOWED_IMAGE_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/bmp",
    "image/tiff",
    "image/webp",
    "image/svg+xml",
];

#[async_trait]
pub trait UserFacadeTrait {
    async fn register(&self, register_model: UserRegisterMultipart) -> Result<UserDetail>;
    async fn login(&self, login_model: UserLogin) -> Result<UserDetail>;
    async fn persist_profile_picture(
        &self,
        profile_picture: NamedTempFile,
        profile_picture_path: String,
    ) -> Result<Option<String>, AppError>;
    async fn replace_profile_picture(
        &self,
        profile_picture: NamedTempFile,
        profile_picture_path: String,
        original_picture_path: Option<String>,
    ) -> Result<Option<String>>;
    async fn validate_password(
        &self,
        password_hash: &Option<String>,
        password: &str,
    ) -> Result<bool>;
    async fn validate_username_exists(&self, username: String) -> Result<(), ValidationError>;
    async fn validate_email_exists(&self, email: String) -> Result<(), ValidationError>;
    async fn validate_picture_mime_type(
        &self,
        profile_picture: &mut NamedTempFile,
    ) -> Result<(), AppError>;
    async fn get_permissions(&self, user_id: i32) -> Result<HashSet<UserRole>>;
    async fn create_profile_picture_folders(
        profile_picture_folder_path: String,
    ) -> anyhow::Result<()>;
    async fn get_user_detail(&self, user_id: i32) -> Result<Option<UserDetail>>;
    async fn update(
        &self,
        user_id: i32,
        user_detail_update: UserDetailUpdate,
    ) -> Result<Option<UserDetail>>;

    async fn update_profile_picture(
        &self,
        user_id: i32,
        profile_picture_update: ProfilePictureUpdate,
    ) -> Result<Option<UserDetail>>;

    async fn change_password(
        &self,
        user_id: i32,
        user_password_update: UserPasswordUpdate,
    ) -> Result<()>;

    async fn delete_user(&self, user_id: i32) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct UserFacade {
    user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,
}

impl UserFacade {
    pub fn new(user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>) -> Self {
        Self { user_repository }
    }

    pub async fn create_profile_picture_folders(
        profile_picture_folder_path: String,
    ) -> anyhow::Result<()> {
        create_dir_if_not_exist(profile_picture_folder_path).await
    }
}

#[async_trait]
impl UserFacadeTrait for UserFacade {
    async fn register(&self, register_form: UserRegisterMultipart) -> Result<UserDetail> {
        let user_register_model = UserRegister::from(&register_form);

        user_register_model
            .validate_model(&UserValidationContext {
                user_repository: Arc::clone(&self.user_repository),
            })
            .await
            .app_error(VALIDATION_ERROR_TEXT)?;

        let password_hash = hash(user_register_model.password.as_str(), DEFAULT_COST)
            .app_error(VALIDATION_ERROR_TEXT)?;

        let mut new_user = User::from(user_register_model);

        new_user.password_hash = Some(password_hash);

        new_user.profile_picture_path = match register_form.profile_picture {
            Some(mut profile_picture) => {
                self.validate_picture_mime_type(&mut profile_picture.file)
                    .await?;

                let profile_picture_file_name = match &profile_picture.file_name {
                    Some(file_name) => file_name.clone(),
                    _ => "".to_string(),
                };

                let unique_file_name = uuid::Uuid::new_v4().to_string();

                let profile_picture_save_path = format!(
                    "{}{}.{}",
                    PROFILE_PICTURE_FOLDER_PATH,
                    unique_file_name,
                    get_file_extension(profile_picture_file_name.clone()).await
                );

                self.persist_profile_picture(profile_picture.file, profile_picture_save_path)
                    .await?;

                format!(
                    "{}{}.{}",
                    "user-images/",
                    unique_file_name,
                    get_file_extension(profile_picture_file_name).await
                )
                .into()
            }
            None => None,
        };

        let created_user_entity = self
            .user_repository
            .create_user(new_user)
            .await
            .app_error("There was an error creating user")?;

        let created_user_model = UserDetail::from(created_user_entity);

        Ok(created_user_model)
    }

    async fn login(&self, login_model: UserLogin) -> Result<UserDetail> {
        let user = self
            .user_repository
            .get_user_by_username(&login_model.username)
            .await?;

        let user = match user {
            Some(user) => user,
            None => return Err(AppError::new(VALIDATION_ERROR_TEXT, BadRequestError)),
        };

        if !self
            .validate_password(&user.password_hash, &login_model.password)
            .await?
        {
            return Err(AppError::new(
                "Invalid username or password",
                BadRequestError,
            ));
        }

        Ok(UserDetail::from(user))
    }

    async fn persist_profile_picture(
        &self,
        profile_picture: NamedTempFile,
        profile_picture_path: String,
    ) -> Result<Option<String>> {
        tokio::fs::copy(profile_picture.path(), &profile_picture_path)
            .await
            .app_error("Failed to save profile picture")?;
        Ok(Some(profile_picture_path))
    }

    async fn replace_profile_picture(
        &self,
        profile_picture: NamedTempFile,
        profile_picture_path: String,
        original_picture_path: Option<String>,
    ) -> Result<Option<String>> {
        tokio::fs::copy(profile_picture.path(), &profile_picture_path)
            .await
            .app_error("Failed to save profile picture")?;

        if let Some(original_picture_path) = original_picture_path {
            let original_file_name = match Path::new(&original_picture_path)
                .file_name()
                .and_then(|name| name.to_str())
            {
                Some(file_name) => file_name,
                None => {
                    return Err(AppError::new(
                        "Original profile picture has been corrupted",
                        AppErrorKind::InternalServerError,
                    ))
                }
            };

            let original_profile_picture_save_path =
                format!("{}{}", PROFILE_PICTURE_FOLDER_PATH, original_file_name);

            info!(
                "old:{}\nnew:{}",
                original_profile_picture_save_path, profile_picture_path
            );

            tokio::fs::remove_file(original_profile_picture_save_path)
                .await
                .app_error("Failed to delete original profile picture")?;
        }

        Ok(Some(profile_picture_path))
    }

    async fn validate_password(
        &self,
        password_hash: &Option<String>,
        password: &str,
    ) -> Result<bool> {
        let password_hash = match password_hash {
            Some(hash) => hash,
            None => {
                return Err(AppError::new(VALIDATION_ERROR_TEXT, BadRequestError));
            }
        };

        Ok(verify(password, password_hash).app_error(VALIDATION_ERROR_TEXT)?)
    }

    async fn validate_username_exists(&self, username: String) -> Result<(), ValidationError> {
        let user = self
            .user_repository
            .get_user_by_username(&username)
            .await
            .map_err(|_| ValidationError::new("An error occurred while validating the username"))?;

        match user {
            Some(_) => Err(ValidationError::new(
                "Username already exists. Please, choose another one",
            )),
            None => Ok(()),
        }
    }

    async fn validate_email_exists(&self, email: String) -> Result<(), ValidationError> {
        let user = self
            .user_repository
            .get_user_by_email(&email)
            .await
            .map_err(|_| ValidationError::new("An error occurred while validating the email"))?;

        match user {
            Some(_) => Err(ValidationError::new(
                "Email already exists. Please, choose another one",
            )),
            None => Ok(()),
        }
    }

    async fn validate_picture_mime_type(
        &self,
        profile_picture: &mut NamedTempFile,
    ) -> Result<(), AppError> {
        let mut profile_picture_content: Vec<u8> = Vec::new();
        profile_picture
            .read_to_end(&mut profile_picture_content)
            .app_error("Profile picture content was not able to be read")?;

        let image_format = image::guess_format(profile_picture_content.as_slice())
            .app_error("Profile picture format was not able to be read")?;

        if !ALLOWED_IMAGE_MIME_TYPES
            .iter()
            .any(|&allowed_mime| allowed_mime == image_format.to_mime_type())
        {
            return Err(AppError::new(
                "Profile picture format not supported",
                BadRequestError,
            ));
        }

        Ok(())
    }

    async fn get_permissions(&self, user_id: i32) -> Result<HashSet<UserRole>> {
        let user = match self.user_repository.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Ok(HashSet::new()),
        };

        let mut user_permissions = HashSet::from([UserRole::Registered]);

        // TODO: check for validity of the membership
        if user.paying_member_id.is_some() {
            user_permissions.insert(UserRole::PayingMember);
        }

        if user.artist_id.is_some() {
            user_permissions.insert(UserRole::Artist);
        }

        Ok(user_permissions)
    }

    async fn create_profile_picture_folders(
        profile_picture_folder_path: String,
    ) -> anyhow::Result<()> {
        Ok(create_dir_if_not_exist(profile_picture_folder_path).await?)
    }

    async fn get_user_detail(&self, user_id: i32) -> Result<Option<UserDetail>> {
        let user = self.user_repository.get_user_by_id(user_id).await?;

        match user {
            Some(user) => Ok(Some(UserDetail::from(user))),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        user_id: i32,
        user_detail_update: UserDetailUpdate,
    ) -> Result<Option<UserDetail>> {
        let user_option = self.user_repository.get_user_by_id(user_id).await?;

        user_detail_update
            .validate()
            .app_error(VALIDATION_ERROR_TEXT)?;

        let mut user = match user_option {
            None => return Err(AppError::from(ValidationError::new("User does not exist"))),
            Some(user) => user,
        };

        let no_changes =
            user.username == user_detail_update.username && user.email == user_detail_update.email;

        if no_changes {
            return Err(AppError::new(
                "Username and email are the same as current values.",
                BadRequestError,
            ));
        }

        if user_detail_update.username != user.username
            && self
                .user_repository
                .get_user_by_username(&user_detail_update.username)
                .await?
                .is_some()
        {
            return Err(AppError::new("Username already exists", BadRequestError));
        }

        if user_detail_update.email != user.email
            && self
                .user_repository
                .get_user_by_email(&user_detail_update.email)
                .await?
                .is_some()
        {
            return Err(AppError::new("Email already exists", BadRequestError));
        }

        user.username = user_detail_update.username;
        user.email = user_detail_update.email;

        let updated_user = self.user_repository.update_user(user).await?;

        match updated_user {
            Some(user) => Ok(Some(UserDetail::from(user))),
            None => Err(AppError::from(ValidationError::new(
                "User update was not successful",
            ))),
        }
    }

    async fn update_profile_picture(
        &self,
        user_id: i32,
        profile_picture_update: ProfilePictureUpdate,
    ) -> Result<Option<UserDetail>> {
        let user_option = self.user_repository.get_user_by_id(user_id).await?;

        let mut user = match user_option {
            None => return Err(AppError::from(ValidationError::new("User does not exist"))),
            Some(user) => user,
        };

        user.profile_picture_path = match profile_picture_update.profile_picture {
            Some(mut profile_picture) => {
                self.validate_picture_mime_type(&mut profile_picture.file)
                    .await?;

                let profile_picture_file_name = match &profile_picture.file_name {
                    Some(file_name) => file_name.clone(),
                    _ => "".to_string(),
                };

                let unique_file_name = uuid::Uuid::new_v4().to_string();

                let profile_picture_save_path = format!(
                    "{}{}.{}",
                    PROFILE_PICTURE_FOLDER_PATH,
                    unique_file_name,
                    get_file_extension(profile_picture_file_name.clone()).await
                );

                self.replace_profile_picture(
                    profile_picture.file,
                    profile_picture_save_path,
                    user.profile_picture_path,
                )
                .await?;

                format!(
                    "{}{}.{}",
                    "user-images/",
                    unique_file_name,
                    get_file_extension(profile_picture_file_name).await
                )
                .into()
            }
            None => None,
        };

        let updated_user_entity = self
            .user_repository
            .update_user(user)
            .await
            .app_error("There was an error updating user")?;

        let updated_user_model = match updated_user_entity {
            Some(updated_user_entity) => UserDetail::from(updated_user_entity),
            None => return Err(AppError::from(ValidationError::new("User does not exist"))),
        };

        Ok(Some(updated_user_model))
    }

    async fn change_password(
        &self,
        user_id: i32,
        user_password_update: UserPasswordUpdate,
    ) -> Result<()> {
        user_password_update
            .validate()
            .app_error(VALIDATION_ERROR_TEXT)?;

        let user = self.user_repository.get_user_by_id(user_id).await?;

        let mut user = match user {
            Some(user) => user,
            None => {
                return Err(AppError::new(
                    "User with provided id does not exist",
                    BadRequestError,
                ))
            }
        };

        if !self
            .validate_password(&user.password_hash, &user_password_update.old_password)
            .await?
        {
            return Err(AppError::new("Old password is invalid", BadRequestError));
        }

        user.password_hash = Some(
            hash(user_password_update.password.as_str(), DEFAULT_COST)
                .app_error(VALIDATION_ERROR_TEXT)?,
        );

        Ok(())
    }

    async fn delete_user(&self, user_id: i32) -> Result<()> {
        let user = self.user_repository.get_user_by_id(user_id).await?;

        if user.is_none() {
            return Err(AppError::new(
                "User with provided id does not exist",
                BadRequestError,
            ));
        };

        self.user_repository.delete_user(user_id).await?;

        Ok(())
    }
}
