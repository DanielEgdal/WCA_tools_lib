use serde::{Deserialize, Serialize};

use super::*;

#[derive(PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub start_date: Date,
    pub number_of_days: usize,
    pub venues: Vec<Venue>
}