use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum UserRole {
    PayingMember,
    Registered,
    Artist,
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}
