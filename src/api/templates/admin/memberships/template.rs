use askama_actix::Template;

#[derive(Template)]
#[template(path = "admin/memberships/index.html")]
pub struct AdminMembershipsTemplate {}
