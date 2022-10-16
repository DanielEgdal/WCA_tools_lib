mod wcif;
mod oauth;
mod wcif_oauth;

use serde::{Deserializer, Deserialize, Serializer};
use serde::de::Error;
pub use wcif::*;
pub use oauth::*;
pub use wcif_oauth::*;

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
    use std::io::Read;
    use std::io::Write;
    use sha2::{Sha512, Digest};

    use crate::OAuth;
    use crate::parse;
    use crate::xor_vecs;

    #[test]
    fn patch() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let test = async {
            let oauth = OAuth::get_auth_with_password(
                    format!("TDg_ARkGANTJB_z0oeUWBVl66a1AYdYAxc-jPJIhSfY"),
                    format!("h0jIi8YkSzJo6U0JRQk-vli21yJ58kuz7_p9-AUyat0"),
                    format!("urn:ietf:wg:oauth:2.0:oob")    
                ).await;
            let mut cont = oauth.get_wcif("test2022").await.unwrap();
            cont.round_iter_mut().next().unwrap().results[0].ranking = Some(1);
            let response = cont.patch(&oauth).await;
            println!("{}", response);
        };
        rt.block_on(test);
    }

    /*#[test]
    fn de() {
        let wcif1 = serde_json::from_str::<Wcif>(include_str!("../wcif.json")).unwrap();
        let json = serde_json::to_string(&wcif1).unwrap();
        println!("{:#?}", json);
        let wcif2 = serde_json::from_str::<Wcif>(&json).unwrap();
        assert_eq!(wcif1, wcif2);
    }*/

    #[test]
    fn hash() {
        let mut hasher = Sha512::new();
        let refresh = std::fs::read("refresh").unwrap();
        std::io::stdout().flush().unwrap();
        let buf = rpassword::read_password().unwrap();
        let now = std::time::Instant::now();
        for _ in 0..1000 {
            hasher.update(buf.trim());
        }
        println!("{:?}", now.elapsed());
        let f = hasher.finalize();
        let g: Result<Vec<u8>, std::io::Error> = f.bytes().collect();
        let k = g.unwrap();
        let s = xor_vecs(k.clone(), refresh);
        println!("{:?}", std::str::from_utf8(&s));
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

#[allow(unused)]
fn xor_vecs(mut hash: Vec<u8>, refresh: Vec<u8>) -> Vec<u8> {
    for i in 0..64 {
        if i < refresh.len() {
            hash[i] ^= refresh[i];
        }
    }
    while hash.last() == Some(&0) {
        hash.pop();
    }
    hash
}