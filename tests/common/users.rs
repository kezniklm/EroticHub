use erotic_hub::business::models::user::UserLogin;

pub const JOHN_ARTIST: TestUser = TestUser {
    username: "JohnArtist",
    password: "12345678",
};

pub const JOHN_NOT_ARTIST: TestUser = TestUser {
    username: "JohnNotArtist",
    password: "12345678",
};

pub const CHARLES_ARTIST: TestUser = TestUser {
    username: "CharlesArtist",
    password: "12345678",
};

pub const JOHN_PAYING: TestUser = TestUser {
    username: "JohnPaying",
    password: "12345678",
};

pub struct TestUser {
    username: &'static str,
    password: &'static str,
}

impl TestUser {
    pub fn get_login_req(&self) -> UserLogin {
        UserLogin {
            username: String::from(self.username),
            password: String::from(self.password),
        }
    }
}
