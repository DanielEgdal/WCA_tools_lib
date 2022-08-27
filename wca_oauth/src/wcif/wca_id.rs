use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::Visitor;

#[derive(Debug, PartialEq)]
pub struct WcaId {
    pub year: u16,
    pub chars: [u8; 4],
    pub id: u8
}

impl<'de> Deserialize<'de> for WcaId {
    fn deserialize<D>(deserializer: D) -> Result<WcaId, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(WcaIdVisitor)
    }
}

struct WcaIdVisitor;

impl<'de> Visitor<'de> for WcaIdVisitor {
    type Value = WcaId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string consisting of 4 integers followed by 4 charachters follwed by 2 integers")
    }
    
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        if v.len() != 10 {
            return Err(E::custom(format!("WcaId too short or too long")));
        }
        let year = &v[0..4];
        let chars = &v[4..8];
        let id = &v[8..10];
        let year = u16::from_str_radix(year, 10).map_err(|_| E::custom(format!("The first four characters of a WcaId is not numerical")))?;
        let id = u8::from_str_radix(id, 10).map_err(|_| E::custom(format!("The last two characters of a WcaId is not numerical")))?;
        Ok(WcaId {
            year,
            chars: chars.as_bytes().try_into().unwrap(),
            id,
        })
    }
}

impl Serialize for WcaId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let str = format!("{:04}{}{:02}",
            self.year,
            self.chars.iter().map(|u|*u as char).collect::<String>(),
            self.id);
        serializer.serialize_str(&str)
    }
}