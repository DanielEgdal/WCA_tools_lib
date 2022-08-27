use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type", content = "level", rename_all = "camelCase")]
pub enum AdvancementCondition {
    Percent(usize),
    Ranking(usize),
    AttemptResult(usize)
}