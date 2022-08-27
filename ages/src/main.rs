use std::{collections::HashMap, ops::Range};

use wca_oauth::Datelike;

const ANONYMOUS: [usize; 0] = [];
const AGE_GROUPS: [Range<i32>; 6] =[0..12, 12..14, 14..18, 18..22, 22..40, 40..100];

fn main() {
    let wcif = wca_oauth::parse(std::fs::read_to_string("wcif.json").unwrap()).unwrap();
    let age_map = wcif.persons_iter()
        .filter(|person| person.registrant_id.is_some())
        .filter(|person| person.country_iso_2 == "DK")
        .filter(|person| !ANONYMOUS.contains(&person.registrant_id.unwrap()))
        .map(|person| {
            let bd = person.birthdate;
            let year = bd.year();
            let month = bd.month();
            let day = bd.day();
            let age = 2022 - year - if month > 9 || (month == 9 && day >= 4) { 1 } else { 0 };
            (person.registrant_id.unwrap(), (age, &person.name))
        })
        .collect::<HashMap<_, _>>();
    wcif.events_iter()
        .find(|event| event.id == "333")
        .expect("It seems that 3x3x3 is not hosted at this competition")
        .rounds
        .iter()
        .flat_map(|round| &round.results)
        .fold(HashMap::new(), |mut acc, result| {
            let id = result.person_id;
            let ranking = result.ranking;
            match ranking {
                Some(v) => {
                    acc.insert(id, v);
                }
                None => ()
            }
            acc
        })
        .iter()
        .filter_map(|(id, ranking)| {
            let (age, name) = age_map.get(id)?;
            Some((age, ranking, id, name))
        })
        .fold(HashMap::new(), |mut acc: HashMap<&Range<i32>, Vec<(usize, usize, &str)>>, (age, ranking, id, name)| {
            let range = AGE_GROUPS.iter().find(|range| range.contains(age)).unwrap();
            acc.entry(range).and_modify(|a| a.push((*ranking, *id, *name))).or_insert(vec![(*ranking, *id, *name)]);
            acc
        })
        .into_iter()
        .for_each(|(group, mut vec)| {
            vec.sort();
            println!("{:?}", group);
            for (ranking, id, name) in vec {
                println!("{}, {}: {}", name, id, ranking);
            }
            println!();
        });
}
