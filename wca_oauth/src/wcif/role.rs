use serde::Deserialize;
use serde::Serialize;
use serde::Deserializer;
use serde::de::Visitor;

#[derive(Debug, PartialEq, Clone)]
pub enum Role {
    Delegate,
    TraineeDelegate,
    Organizer,
    Other(String)
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Role, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(RoleVisitor)
    }
}

struct RoleVisitor;

impl<'de> Visitor<'de> for RoleVisitor {
    type Value = Role;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a role enum variant")
    }
    
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(match v {
            "delegate" => Role::Delegate,
            "trainee-delegate" => Role::TraineeDelegate,
            "organizer" => Role::Organizer,
            v => Role::Other(v.to_string())
        })
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(match self {
            Role::Delegate => "delegate",
            Role::TraineeDelegate => "trainee-delegate",
            Role::Organizer => "organizer",
            Role::Other(v) => v
        })
    }
}
