use std::{collections::HashMap, io::Write};
use std::fs::File;
use crate::wcif::get_round_json;
use scorecard_to_pdf::{Scorecard, TimeLimit, scorecards_to_pdf, Language};
use wca_oauth::WcifOAuth;
use scorecard_to_pdf::Return;

#[derive(Clone)]
pub struct Stages {
    data: Vec<(Option<String>, usize)>
}

impl Stages {
    pub fn new() -> Stages {
        Stages { data: vec![] }
    }

    pub fn add_stage(&mut self, ident: Option<String>, size: usize) {
        self.data.push((ident, size));
    }
}

pub fn save_pdf(data: Return, competition: &str) -> std::io::Result<()> {
    let (data, name) = match data {
        Return::Pdf(b) => (b, ".pdf"),
        Return::Zip(b) => (b, ".zip")
    };
    let mut file = File::create(competition.split_ascii_whitespace().collect::<String>() + "_scorecards" + name)?;
    file.write_all(&data)?;
    Ok(())
}

pub fn stage_ident(station: Option<usize>, stages: &Option<Stages>) -> Option<&str> {
    let mut station = station?;
    let org = station;
    let stages = stages.as_ref()?;
    for (ident, size) in stages.data.iter() {
        if station <= *size {
            return match ident {
                Some(v) => Some(&v),
                None => None
            }
        }
        else {
            station -= size;
        }
    }
    panic!("Invalid station number given: {}", org)
}

pub fn run(groups_csv: &str, limit_csv: &str, competition: &str, language: Language, wcif: Option<&mut WcifOAuth>, stages: Option<Stages>) -> Return {
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
                stage: stage_ident(station, &stages)
            }
        })
        .collect::<Vec<_>>();
    //Sort scorecards by event, round, group, station (Definition order) 
    k.sort();


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

    //Generate pdf
    scorecards_to_pdf(k, competition, &map, &limits, language, wcif).unwrap_or_else(|e| e)
}

pub async fn run_from_wcif(wcif: &mut WcifOAuth, event: &str, round: usize, groups: Vec<Vec<usize>>, stages: &Option<Stages>) -> Result<Return, Return> {
    let (map, limit, competition) = crate::wcif::get_scorecard_info_for_round(wcif, event, round);

    //Unwrap should not fail as the existence of this round is already confirmed at this point.
    get_round_json(wcif, event, round).unwrap().scramble_set_count = groups.len();

    let mut limits = HashMap::new();
    limits.insert(event, limit);

    let k = groups.into_iter()
        .enumerate()
        .map(|(n, group)|{
            let size = group.len();
            let (no_of_stages, stage_capacity) = if let Some(stages) = stages {
                let mut size_left = size;
                let mut no_of_stages = 0;
                let mut stage_capacity = 0;
                for stage in stages.data.iter() {
                    no_of_stages += 1;
                    stage_capacity += stage.1;
                    if size_left > stage.1 {
                        size_left -= stage.1;
                    }
                    else {
                        break;
                    }
                }
                (no_of_stages, stage_capacity)
            }
            else {
                (1, size)
            };
            let over_capacity = stage_capacity - size;
            let over_per_stage = over_capacity / no_of_stages;
            let leftover = over_capacity - over_per_stage * no_of_stages;
            let mut current_offset = 0;
            let mut remaining_on_stage = if let Some(stages) = stages {
                stages.data[0].1 - over_per_stage
            }
            else {
                size
            };
            let mut current_stage = 0;
            group.into_iter()
                .enumerate()
                .map(move |(station, id)|{
                    if remaining_on_stage == 0 {
                        current_offset += over_per_stage + if  current_stage >= no_of_stages - leftover { 1 } else { 0 };
                        current_stage += 1;
                        remaining_on_stage = stages.as_ref().unwrap().data[current_stage].1 - over_per_stage - if current_stage >= no_of_stages - leftover { 1 } else { 0 };
                    }
                    remaining_on_stage -= 1;
                    Scorecard {
                        event,
                        round,
                        group: n + 1,
                        station: Some(station + 1 + current_offset),
                        id,
                        stage: stage_ident(Some(station + 1 + current_offset), &stages)
                    }
                })
        }).flatten()
        .collect::<Vec<_>>();
    
    scorecards_to_pdf(k, &competition, &map, &limits, Language::english(), Some(wcif))
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
