use std::{collections::HashMap, io::Write};
use std::fs::File;
use crate::ScorecardOrdering;
use crate::wcif::get_round_json;
use scorecard_to_pdf::{Scorecard, TimeLimit, scorecards_to_pdf, Language};
use wca_oauth::WcifContainer;
use scorecard_to_pdf::Return;

#[derive(Clone)]
pub struct Stages {
    pub(crate) no: u32,
    pub(crate) capacity: u32,
}

impl Stages {
    pub fn new(no: u32, capacity: u32) -> Stages {
        Stages { no, capacity }
    }
}

pub fn save_pdf(data: Return, competition: &str, prefix: &str) -> std::io::Result<()> {
    let (data, name) = match data {
        Return::Pdf(b) => (b, ".pdf"),
        Return::Zip(b) => (b, ".zip")
    };
    let file_name = format!("{prefix}{}_scorecards{name}", competition.split_ascii_whitespace().collect::<String>());
    let mut file = File::create(file_name)?;
    file.write_all(&data)?;
    Ok(())
}

pub(crate) fn run(groups_csv: &str, limit_csv: Option<String>, competition: &str, language: Language, stages: Stages, compare: ScorecardOrdering) -> Return {
    let mut groups_csv = groups_csv.lines();
    //Header describing csv file formatting. First two are fixed and therfore skipped.
    //Unwrap cannot fail because the first element of lines always exists, although skip can lead
    //to panic later when used.
    let header = groups_csv.next().unwrap().split(",").skip(2);
    let mut map = HashMap::new();
    let mut k = groups_csv
        //Filter off empty lines. Fixes annoying EOF issues.
        .filter(|x|*x != "")
        //Map each person to each event they compete in.
        //Enumerate for panic messages
        .enumerate()
        .map(|(line, person)|{
            let mut iter = person.split(",");
            let name = match iter.next() {
                None => panic!("Line {} in csv missing name", line + 2),
                Some(v) => v
            };
            let id = match iter.next() {
                None => panic!("Line {} in csv missing id", line + 2),
                Some(v) => v
            };
            let id = match usize::from_str_radix(id, 10) {
                Err(_) => panic!("Id for {} in line {} is not a positive integer", name, line + 2),
                Ok(v) => v
            };
            //Insert the competitor into the id to name map.
            map.insert(id, name.to_string());
            //Zipping with header (clone) to know the order of events.
            iter.zip(header.clone())
                .filter_map(move |(asign, event)|{
                //Test whether competitor is assigned.
                if asign == "" {
                    return None
                }
                else {
                    let mut info = asign.split(";");
                    let pre_group = info.next()?;
                    let group = match usize::from_str_radix(pre_group, 10) {
                        Err(_) => panic!("Group number for event {} in line {} is nut a positive integer", event, line + 2),
                        Ok(v) => v
                    };
                    let station = info.next().map(|v| match usize::from_str_radix(v, 10) {
                        Err(_) => panic!("Station number for event {} in line {} is not a positive integer", event, line + 2),
                        Ok(v) => v
                    });
                    Some((id, event, group, station))
                }
            })
        })
        .flatten()
        .map(|(id, event, group, station)|{
            Scorecard {
                id,
                group,
                round: 1,
                station,
                event,
                stage: station.map(|x| (x as u32 - 1) / stages.capacity),
            }
        })
        .collect::<Vec<_>>();
    //Sort scorecards by event, round, group, station (Definition order) 
    compare.sort_slice(&mut k);

    let limits = match &limit_csv {
        Some(limit_csv) => {
            //Parse time limits
            let mut limit = limit_csv.lines();
            //Header cannot fail because first in lines
            let event_list = limit.next().unwrap().split(",");
            let limit_data = match limit.next() {
                None => panic!("No time limits given in time limit csv"),
                Some(v) => v
            }.split(",");

            let mut limits = HashMap::new();
            limit_data.zip(event_list).for_each(|(x, event)|{
                let mut iter = x.split(";");
                let v = match iter.next() {
                    None => {
                        limits.insert(event, TimeLimit::None);
                        return;
                    }
                    Some(v) => v,
                };
                match v {
                    "T" => limits.insert(event, TimeLimit::Single(usize_from_iter(&mut iter))),
                    "C" => limits.insert(event, TimeLimit::Cumulative(usize_from_iter(&mut iter))),
                    "K" => limits.insert(event, TimeLimit::Cutoff(usize_from_iter(&mut iter), usize_from_iter(&mut iter))),
                    "S" => limits.insert(event, TimeLimit::SharedCumulative(usize_from_iter(&mut iter), iter.map(|x|x.to_string()).collect::<Vec<_>>())),
                    "M" => limits.insert(event, TimeLimit::Multi),
                    _ => panic!("Malformatted time limit for event: {}", event)
                };
            });
            limits
        },
        None => HashMap::new(),
    };
    

    //Generate pdf
    scorecards_to_pdf(k, competition, &map, &limits, language)
}

pub(crate) fn run_from_wcif(wcif: &mut WcifContainer, event: &str, round: usize, groups: Vec<Vec<(usize, usize)>>, stages: &Stages, compare: ScorecardOrdering) -> Return {
    let (map, limit, competition) = crate::wcif::get_scorecard_info_for_round(wcif, event, round);

    //Unwrap should not fail as the existence of this round is already confirmed at this point.
    get_round_json(wcif, event, round).unwrap().scramble_set_count = groups.len();
    let mut limits = HashMap::new();
    limits.insert(event, limit);

    let mut k = groups.into_iter()
        .zip(1..)
        .map(|(group, n)|{
            group.into_iter()
                .map(move |(id, station)|{
                    Scorecard {
                        event,
                        round,
                        group: n,
                        station: Some(station),
                        id,
                        stage: Some((station as u32 - 1) / stages.capacity),
                    }
                })
        }).flatten()
        .collect::<Vec<_>>();

    compare.sort_slice(&mut k);
    
    scorecards_to_pdf(k, &competition, &map, &limits, Language::english())
}

fn usize_from_iter<'a, I>(iter: &mut I) -> usize where I: Iterator<Item = &'a str> {
    match usize::from_str_radix(match iter.next() {
        None => panic!("Malformatted input file. Missing data, where integer was expected"),
        Some(v) => v
    }, 10) {
        Err(_) => panic!("Malformatted input file. Expected positive integer, but received other charachters"),
        Ok(v) => v
    }
}
