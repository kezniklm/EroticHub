use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use crate::business::models::user::{UserRole, UserSessionData};
use actix_identity::error::GetIdentityError;
use actix_identity::Identity;
use actix_session::{Session, SessionGetError};
use actix_web::dev::{Payload, ServiceRequest};
use actix_web::web::Data;
use actix_web::{Error, FromRequest};
use std::collections::HashSet;
use std::str::FromStr;

pub async fn extract(req: &ServiceRequest) -> Result<HashSet<UserRole>, Error> {
    let identity = match Identity::from_request(req.request(), &mut Payload::None).await {
        Ok(identity) => identity,
        Err(_) => return Ok(HashSet::new()),
    };

    let user_facade = match req.app_data::<Data<UserFacade>>() {
        Some(facade) => facade,
        None => return Ok(HashSet::new()),
    };

    let user_id = match identity.id()?.parse::<i32>() {
        Ok(user_id) => user_id,
        Err(_) => return Ok(HashSet::new()),
    };

    match user_facade.get_permissions(user_id).await {
        Ok(permissions) => Ok(permissions),
        Err(_) => Ok(HashSet::new()),
    }
}

pub trait AsInteger {
    fn id_i32(&self) -> Result<i32, GetIdentityError>;
}

impl AsInteger for Identity {
    fn id_i32(&self) -> Result<i32, GetIdentityError> {
        self.id().map(|id| {
            let id = i32::from_str(&id);
            match id {
                Ok(id) => Ok(id),
                Err(_) => Err(GetIdentityError::SessionGetError(SessionGetError::from(
                    anyhow::Error::msg("Failed to extract session ID"),
                ))),
            }
        })?
    }
}

pub trait AsIntegerOptional {
    fn id_i32(self) -> Option<i32>;
}

impl AsIntegerOptional for Option<Identity> {
    fn id_i32(self) -> Option<i32> {
        self.map(|id| id.id_i32().ok())?
    }
}

pub trait IsRole {
    fn is_artist(&self) -> bool;
    fn is_paying_member(&self) -> bool;
    fn is_registered(&self) -> bool;

    fn extract_and_check(
        &self,
        session_data: Result<Option<UserSessionData>, SessionGetError>,
        expected_role: UserRole,
    ) -> bool {
        if let Ok(session_data) = session_data {
            return match session_data {
                None => false,
                Some(data) => data.roles.contains(&expected_role),
            };
        };
        false
    }
}

impl IsRole for Session {
    fn is_artist(&self) -> bool {
        let session_data = self.get::<UserSessionData>("user_session_data");
        self.extract_and_check(session_data, UserRole::Artist)
    }

    fn is_paying_member(&self) -> bool {
        let session_data = self.get::<UserSessionData>("user_session_data");
        self.extract_and_check(session_data, UserRole::PayingMember)
    }

    fn is_registered(&self) -> bool {
        let session_data = self.get::<UserSessionData>("user_session_data");
        self.extract_and_check(session_data, UserRole::Registered)
    }
}
