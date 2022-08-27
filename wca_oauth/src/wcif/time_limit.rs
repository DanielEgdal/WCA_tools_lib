use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimeLimit {
    pub centiseconds: usize,
    pub cumulative_round_ids: Vec<String>
}