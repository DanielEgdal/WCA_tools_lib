use serde::{Deserialize, Serialize};

use super::*;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub registrant_id: Option<usize>,
    pub name: String,
    pub wca_user_id: usize,
    pub wca_id: Option<WcaId>,
    pub country_iso_2: String,
    pub gender: char,
    pub birthdate: serde_with::chrono::NaiveDate,
    pub email: String,
    pub avatar: Option<Avatar>,
    pub roles: Vec<Role>,
    pub registration: Option<Registration>,
    pub assignments: Vec<Assignment>,
    pub personal_bests: Vec<PersonalBest>,
}