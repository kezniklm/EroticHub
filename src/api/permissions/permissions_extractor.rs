use crate::business::facades::user::{UserFacade, UserFacadeTrait};
use actix_session::Session;
use actix_web::dev::ServiceRequest;
use actix_web::web::Data;
use actix_web::{Error, HttpMessage};
use std::collections::HashSet;

pub async fn extract(req: &ServiceRequest) -> Result<HashSet<String>, Error> {
    let session = match req.extensions().get::<Session>() {
        Some(session) => session.clone(),
        None => return Ok(HashSet::new()),
    };

    let user_id_string = match session.get::<String>("user_id") {
        Ok(id) => match id {
            Some(id) => id,
            None => return Ok(HashSet::new()),
        },
        Err(_) => return Ok(HashSet::new()),
    };

    let user_id = match user_id_string.parse::<i32>() {
        Ok(parsed_id) => parsed_id,
        Err(_) => return Ok(HashSet::new()),
    };

    let user_facade = match req.app_data::<Data<UserFacade>>() {
        Some(facade) => facade,
        None => return Ok(HashSet::new()),
    };

    match user_facade.get_permissions(user_id).await {
        Ok(permissions) => Ok(permissions.into_iter().collect()),
        Err(_) => Ok(HashSet::new()),
    }
}
