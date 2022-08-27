use serde::{Deserialize, Serialize};

use super::*;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Event {
    pub id: String,
    pub rounds: Vec<Round>,
    pub competitor_limit: Option<usize>,
    pub qualification: Option<serde_json::Value>,
    pub extensions: Vec<serde_json::Value>
}