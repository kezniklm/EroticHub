use askama_actix::Template;

use crate::business::models::deal::DealModel;

#[derive(Template)]
#[template(path = "admin/edit_deal/index.html")]
pub struct AdminEditDealTemplate {
    pub deal: Option<DealModel>,
}
