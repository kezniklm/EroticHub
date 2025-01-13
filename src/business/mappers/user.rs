use crate::business::models::user::{UserDetail, UserRegister, UserRegisterMultipart};
use crate::persistence::entities::user::User;

impl From<User> for UserDetail {
    fn from(user: User) -> Self {
        UserDetail {
            id: user.id,
            username: user.username,
            email: user.email,
            profile_picture_path: user.profile_picture_path,
            artist_id: user.artist_id,
            paying_member_id: user.paying_member_id,
        }
    }
}

impl From<UserDetail> for User {
    fn from(user_detail: UserDetail) -> Self {
        User {
            id: user_detail.id,
            username: user_detail.username,
            password_hash: None,
            email: user_detail.email,
            profile_picture_path: user_detail.profile_picture_path,
            artist_id: user_detail.artist_id,
            paying_member_id: user_detail.paying_member_id,
        }
    }
}

impl From<UserRegister> for User {
    fn from(user_register: UserRegister) -> Self {
        User {
            id: -1,
            username: user_register.username,
            password_hash: None,
            email: user_register.email,
            profile_picture_path: None,
            artist_id: None,
            paying_member_id: None,
        }
    }
}

impl From<&UserRegisterMultipart> for UserRegister {
    fn from(user_register: &UserRegisterMultipart) -> Self {
        UserRegister {
            username: user_register.username.to_string(),
            password: user_register.password.to_string(),
            password2: user_register.password2.to_string(),
            email: user_register.email.to_string(),
        }
    }
}
