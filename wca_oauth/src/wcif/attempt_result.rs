use serde::Deserialize;
use serde::Serialize;
use serde::Deserializer;
use serde::de::Visitor;

#[derive(Clone, Debug, PartialEq)]
pub enum AttemptResult {
    DNF,
    DNS,
    Skip,
    Ok(usize)
}

impl<'de> Deserialize<'de> for AttemptResult {
    fn deserialize<D>(deserializer: D) -> Result<AttemptResult, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(AttemptResultVisitor)
    }
}

struct AttemptResultVisitor;

impl<'de> Visitor<'de> for AttemptResultVisitor {
    type Value = AttemptResult;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an attemptresult")
    }
    
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(match v {
            -1 => AttemptResult::DNF,
            -2 => AttemptResult::DNS,
            0 => AttemptResult::Skip,
            _ => AttemptResult::Ok(v as usize)
        })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
            Ok(match v {
                0 => AttemptResult::Skip,
                _ => AttemptResult::Ok(v as usize)
            })
    }
}

impl Serialize for AttemptResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        match self {
            AttemptResult::Ok(v) => return v.serialize(serializer),
            _ => ()
        }
        serializer.serialize_i64(match self {
            AttemptResult::DNF => -1,
            AttemptResult::DNS => -2,
            AttemptResult::Skip => 0,
            AttemptResult::Ok(v) => *v as i64,
        })
    }
}