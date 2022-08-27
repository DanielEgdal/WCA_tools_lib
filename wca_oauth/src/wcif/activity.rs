use serde::{Deserialize, Serialize};
use super::DateTime;

#[derive(PartialEq, Debug, Deserialize, Serialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: usize,
    pub name: String,
    pub activity_code: String,
    #[serde(deserialize_with = "crate::de_date_time", serialize_with = "crate::ser_date_time")]
    pub start_time: DateTime,
    #[serde(deserialize_with = "crate::de_date_time", serialize_with = "crate::ser_date_time")]
    pub end_time: DateTime,
    pub child_activities: Vec<Activity>,
    pub scramble_set_id: Option<usize>,
    pub extensions: Vec<serde_json::Value>
}

impl Activity {
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start_time < other.end_time && other.start_time < self.end_time && self != other
    }

    pub fn overlaps_any<'a>(&self, other: impl IntoIterator<Item = &'a Self> + 'a) -> bool {
        other.into_iter().any(|other|self.overlaps(other))
    }
}