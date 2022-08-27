use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Assignment {
    pub activity_id: usize,
    pub assignment_code: AssignmentCode,
    pub station_number: Option<usize>
}

use serde::Deserializer;
use serde::de::Visitor;

#[derive(Debug, PartialEq)]
pub enum AssignmentCode {
    Competitor,
    Judge,
    Scrambler,
    Runner,
    DataEntry,
    Announcer,
    Other(String)
}

impl<'de> Deserialize<'de> for AssignmentCode {
    fn deserialize<D>(deserializer: D) -> Result<AssignmentCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AssignmentCodeVisitor)
    }
}

struct AssignmentCodeVisitor;

impl<'de> Visitor<'de> for AssignmentCodeVisitor {
    type Value = AssignmentCode;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a role enum variant")
    }
    
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(match v {
            "competitor" => AssignmentCode::Competitor,
            "staff-judge" => AssignmentCode::Judge,
            "staff-scrambler" => AssignmentCode::Scrambler,
            "staff-runner" => AssignmentCode::Runner,
            "staff-dataentry" => AssignmentCode::DataEntry,
            "staff-announcer" => AssignmentCode::Announcer,
            v => AssignmentCode::Other(v.to_string())
        })
    }
}

impl Serialize for AssignmentCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(match self {
            AssignmentCode::Competitor => "competitor",
            AssignmentCode::Judge => "staff-judge" ,
            AssignmentCode::Scrambler => "staff-scrambler" ,
            AssignmentCode::Runner => "staff-runner" ,
            AssignmentCode::DataEntry => "staff-dataentry" ,
            AssignmentCode::Announcer => "staff-announcer" ,
            AssignmentCode::Other(v) => &v

        })
    }
}