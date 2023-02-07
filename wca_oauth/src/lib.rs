mod wcif;
mod oauth;
mod competition;
mod wcif_oauth;

use serde::{Deserializer, Deserialize, Serializer};
use serde::de::Error;
pub use wcif::*;
pub use oauth::*;
pub use wcif_oauth::*;
pub use competition::*;

pub use serde_with::chrono::{NaiveDateTime as DateTime, NaiveDate as Date, NaiveTime as Time, Datelike};

fn de_date_time<'de, D>(deserializer: D) -> std::result::Result<DateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    if s.chars().last().unwrap() == 'Z' {
        Ok(serde_json::from_str(&format!("\"{}\"", &s[0..s.len() - 1])).unwrap())
    }
    else {
        Err(D::Error::custom(s))
    }
}

fn ser_date_time<S>(date_time: &DateTime, serializer: S) -> std::result::Result<S::Ok, S::Error> 
where 
    S: Serializer 
{
    let str = serde_json::to_string(&date_time).unwrap();
    serializer.serialize_str(&format!("{}Z", &str[1..str.len() - 1]))
}

#[cfg(test)]
mod test {
    use crate::{parse, Wcif};

    #[test]
    fn de() {
        let wcif1 = serde_json::from_str::<Wcif>(include_str!("../wcif.json")).unwrap();
        let json = serde_json::to_string(&wcif1).unwrap();
        println!("{:#?}", json);
        let wcif2 = serde_json::from_str::<Wcif>(&json).unwrap();
        assert_eq!(wcif1, wcif2);
    }

    #[test]
    fn overlapping() {
        let cont = parse(std::fs::read_to_string("wcif.json").unwrap()).unwrap();
        let k = cont.overlapping_activities();
        for (a, b) in k {
            println!("{:?}, {:?}", a.activity_code, b.activity_code);
        }
    }
}
