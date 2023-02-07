use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Competition {
    id: String,
    name: String,
    registration_open: String,
    registration_close: String,
    announced_at: Option<String>,
    start_date: String,
    end_date: String,
    competitor_limit: Option<u64>,
    cancelled_at: Option<String>,
    url: String,
    website: String,
    short_name: String,
    city: String,
    venue_adress: String,
    latitude_degrees: f64,
    longitude_degrees: f64,
    country_iso2: String,
    event_ids: Vec<String>,
    delegates: Vec<serde_json::Value>,
    organizers: Vec<serde_json::Value>,
}

impl Competition {
    pub fn from_json(json: &str) -> Vec<Competition> {
        serde_json::from_str(json).unwrap()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
