use serde::{Deserialize, Serialize};

use super::*;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Round {
    pub id: String,
    pub format: char,
    pub time_limit: Option<TimeLimit>,
    pub cutoff: Option<Cutoff>,
    pub advancement_condition: Option<AdvancementCondition>,
    pub results: Vec<Result>,
    pub scramble_set_count: usize,
    pub extensions: Vec<serde_json::Value>,
}