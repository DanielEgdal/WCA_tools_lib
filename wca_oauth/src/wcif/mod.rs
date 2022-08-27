use serde::{Deserialize, Serialize};

mod activity;
mod advancement_condition;
mod assignment;
mod attempt_result;
mod avatar;
mod cutoff;
mod event;
mod person;
mod personal_best;
mod registration;
mod result;
mod role;
mod room;
mod round;
mod schedule;
mod time_limit;
mod venue;
mod wca_id;

pub use activity::*;
pub use advancement_condition::*;
pub use assignment::*;
pub use attempt_result::*;
pub use avatar::*;
pub use cutoff::*;
pub use event::*;
pub use person::*;
pub use personal_best::*;
pub use registration::*;
pub use result::*;
pub use role::*;
pub use room::*;
pub use round::*;
pub use schedule::*;
pub use time_limit::*;
pub use venue::*;
pub use wca_id::*;
pub use super::{Date, DateTime};

use crate::WcifContainer;

pub type WcifResult = std::result::Result<WcifContainer, WcifError>;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Wcif {
    pub format_version: String,
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub persons: Vec<Person>,
    pub events: Vec<Event>,
    pub schedule: Schedule,
    pub competitor_limit: Option<usize>,
    pub extensions: Vec<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct WcifError {
    pub error: String
}

pub fn parse(json: String) -> WcifResult {
    serde_json::from_str(&json).map(|wcif|WcifContainer::new(wcif)).map_err(|_| serde_json::from_str(&json).unwrap())
}