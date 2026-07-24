use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JobAiSummary {
    pub availability: String,
    pub pay_rate: String,
    pub responsibilities: String,
    pub eligibility: String,
    #[serde(rename = "NOTE")]
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Job {
    pub name: String,
    pub id: String,
    pub posting_date: Option<NaiveDate>,
    pub location_city: String,
    pub location_country: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub ai_summary: Option<JobAiSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppState {
    pub mode: String, // "Default" or "Debug"
    pub pause_time: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub dashboard_msg_id: Option<u64>,
    #[serde(default)]
    pub consecutive_errors: u32,
}
