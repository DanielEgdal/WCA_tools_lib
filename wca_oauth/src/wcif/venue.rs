use serde::{Deserialize, Serialize};

use super::*;

#[derive(PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Venue {
    pub id: usize,
    pub name: String,
    pub latitude_microdegrees: i64,
    pub longitude_microdegrees: i64,
    pub country_iso_2: String,
    pub timezone: String,
    pub rooms: Vec<Room>,
    pub extensions: Vec<serde_json::Value>
}