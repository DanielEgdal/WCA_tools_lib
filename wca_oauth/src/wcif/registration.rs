use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub wca_registration_id: usize,
    pub event_ids: Vec<String>,
    pub status: String,
    pub guests: usize,
    pub comments: String
}