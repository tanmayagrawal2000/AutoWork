use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Job {
    pub name: String,
    pub id: String,
    pub posting_date: Option<NaiveDate>,
    pub location_city: String,
    pub location_country: String,
}
