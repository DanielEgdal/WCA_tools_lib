use serde::{Deserialize, Serialize};

use crate::AttemptResult;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub person_id: usize,
    pub ranking: Option<usize>,
    pub attempts: Vec<Attempt>,
    pub best: AttemptResult,
    pub average: AttemptResult,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub result: AttemptResult,
    pub reconstruction: Option<serde_json::Value>,
}