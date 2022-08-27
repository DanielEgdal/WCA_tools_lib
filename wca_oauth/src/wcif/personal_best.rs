use serde::{Deserialize, Serialize};

use crate::AttemptResult;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PersonalBest {
    pub event_id: String,
    pub best: AttemptResult,
    #[serde(rename = "type")]
    pub t: String,
    pub world_ranking: usize,
    pub continental_ranking: usize,
    pub national_ranking: usize
}