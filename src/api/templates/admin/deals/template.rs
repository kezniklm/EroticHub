use askama_actix::Template;

use crate::business::models::deal::DealModel;

#[derive(Template)]
#[template(path = "admin/deals/index.html")]
pub struct AdminDealsTemplate {
    pub deals: Vec<DealModel>,
}
