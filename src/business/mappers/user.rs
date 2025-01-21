use crate::business::models::user_detail::{UserDetail, Username};
use crate::business::models::user_list::UserList;
use crate::persistence::entities::user::{User, UserName};

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

impl From<UserName> for Username {
    fn from(user: UserName) -> Self {
        Username {
            id: user.id,
            username: user.username,
        }
    }
}
