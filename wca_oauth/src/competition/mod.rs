use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Competition {
    id: String,
    name: String,
}

impl Competition {
    pub fn from_json(json: &str) -> Vec<Competition> {
        serde_json::from_str(json).unwrap()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
