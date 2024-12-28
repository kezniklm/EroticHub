use crate::business::models::user_detail::UserDetail;
use crate::business::models::user_list::UserList;
use crate::business::models::user_register::UserRegister;
use crate::persistence::entities::user::User;

impl From<User> for UserDetail {
    fn from(user: User) -> Self {
        UserDetail {
            id: user.id,
            username: user.username,
            password_hash: user.password_hash,
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
            password_hash: user_detail.password_hash,
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
            id: user_register.id,
            username: user_register.username,
            password_hash: None,
            email: user_register.email,
            profile_picture_path: user_register.profile_picture_path,
            artist_id: None,
            paying_member_id: None,
        }
    }
}

impl From<User> for UserList {
    fn from(user: User) -> Self {
        UserList {
            id: user.id,
            username: user.username,
            email: user.email,
            profile_picture_path: user.profile_picture_path,
        }
    }
}
