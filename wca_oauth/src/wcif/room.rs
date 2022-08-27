use serde::{Deserialize, Serialize};

use super::*;

#[derive(PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: usize,
    pub name: String,
    pub color: String,
    pub activities: Vec<Activity>,
    pub extensions: Vec<serde_json::Value>
}