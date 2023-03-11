use std::collections::HashMap;

use wca_oauth::*;

use scorecard_to_pdf::TimeLimit;

pub fn get_rounds(wcif: &mut WcifContainer) -> Vec<(String, usize)> {
    wcif.events_iter()
        .map(|event|event.rounds
            .iter()
            .map(|round|round.id.to_string()))
        .flatten()
        .map(|str|{
            let mut iter = str.split("-r");
            (iter.next().unwrap().to_string(), usize::from_str_radix(iter.next().unwrap(), 10).unwrap())
        })
        .collect()
}

pub fn get_scorecard_info_for_round(wcif: &mut WcifContainer, event: &str, round: usize) -> (HashMap<usize, String>, TimeLimit, String) {
    let id_map = get_id_map(wcif);
    let time_limit = get_time_limit(wcif, event, round);
    (id_map, time_limit, wcif.get().name.clone())
}

pub fn get_time_limit(wcif: &mut WcifContainer, event: &str, round: usize) -> TimeLimit {
    let round_json = get_round_json(wcif, event, round).unwrap();
    match &round_json.time_limit {
        None => TimeLimit::Multi,
        Some(v) => {
            match round_json.cutoff {
                None => match v.cumulative_round_ids.len() {
                    0 => TimeLimit::Single(v.centiseconds),
                    1 => TimeLimit::Cumulative(v.centiseconds),
                    _ => TimeLimit::SharedCumulative(v.centiseconds, v.cumulative_round_ids.clone())
                }
                Some(ref c) => TimeLimit::Cutoff(v.centiseconds, if let AttemptResult::Ok(v) = c.attempt_result {v} else {unreachable!()})
            }
        }
    }
}

pub fn wca_live_get_competitors_for_round(wcif: &mut WcifContainer, event: &str, round: usize) -> (Vec<usize>, HashMap<usize, String>) {
    let id_map = get_id_map(wcif);
    // Get the previous round, so we can sort people correctly by speed.
    let round_json_prev = get_round_json(wcif, event, round - 1);
    let advancement_ids_prev: HashMap<_, _> = match round_json_prev {
        Some(v) => {
            v.results.iter().map(|r| (r.person_id, r.ranking.expect("This is a previous round, so there is a ranking"))).collect()
        },
        None => {
            HashMap::new()
        }
    };

    // Now actually get those who proceeded
    let round_json = get_round_json(wcif, event, round).expect("Round should exist");
    let mut advancement_ids = wca_live_get_advancement_ids(&round_json);
    if !advancement_ids.is_empty(){
        if !advancement_ids_prev.is_empty(){
            advancement_ids.sort_by_key(|&x| advancement_ids_prev.get(&x).unwrap_or(&std::usize::MAX));
            (advancement_ids, id_map)
        }
        else{
            (advancement_ids, id_map)
        }
    }
    else{
        get_competitors_for_round(wcif,event,round)
    }
}

pub fn get_competitors_for_round(wcif: &mut WcifContainer, event: &str, round: usize) -> (Vec<usize>, HashMap<usize, String>) {
    let id_map = get_id_map(wcif);
    let round_json = get_round_json(wcif, event, round - 1);
    let advancement_ids = match round_json {
        Some(v) => get_advancement_ids(v, &v.advancement_condition),
        None => {
            wcif.persons_iter().filter_map(|p|{
                let reg = p.registration.as_ref()?;
                if reg.status == format!("accepted") && reg.event_ids.contains(&event.to_string()) {
                    Some(p.registrant_id.unwrap())
                } else { None }
            }).collect()
        }
    };
    (advancement_ids, id_map)
}

pub(crate) fn get_round_json<'a>(wcif: &'a mut WcifContainer, event: &str, round: usize) -> Option<&'a mut Round> {
    let activity_id = format!("{}-r{}", event, round);
    wcif.round_iter_mut().find(|round| round.id == activity_id)
}

fn get_advancement_amount(round: &Round, advancement_condition: &Option<AdvancementCondition>) -> Option<usize> {
    let number_of_competitors = round.results.len();
    match advancement_condition {
        None => None,
        Some(v) => Some( match v {
            AdvancementCondition::Percent(level) => number_of_competitors * level / 100,
            AdvancementCondition::Ranking(level) => *level,
            AdvancementCondition::AttemptResult(level) => {
                let mut intermediate = round.results.iter().collect::<Vec<_>>();
                intermediate.sort_by_key(|r| r.ranking);
                let x = intermediate.into_iter().enumerate().find(|(_, result)|{
                    match result.average {
                        AttemptResult::DNF | AttemptResult::DNS | AttemptResult::Skip => true,
                        AttemptResult::Ok(average) => average as usize > *level
                    }
                }).map(|(x, _)| x);
                let percent = get_advancement_amount(round, &Some(AdvancementCondition::Percent(75))).unwrap();
                match x {
                    Some(v) if v < percent => v,
                    _ => percent
                }
            }
        })
    }
}

pub fn get_id_map(wcif: &WcifContainer) -> HashMap<usize, String> {
    wcif.persons_iter().filter_map(|p| p.registrant_id.map(|v|(v, p.name.clone()))).collect()
}

fn wca_live_get_advancement_ids(round: &Round) -> Vec<usize> {
    let advacenment_ids = round.results.iter().map(|f|
    f.person_id).collect();
    advacenment_ids
}

fn get_advancement_ids(round: &Round, advancement_condition: &Option<AdvancementCondition>) -> Vec<usize> {
    let advancement_amount = get_advancement_amount(round, advancement_condition);
    match advancement_amount {
        None => return vec![],
        Some(advancement_amount) => {
            let mut intermediate = round.results.iter().collect::<Vec<_>>();
            intermediate.sort_by_key(|r| r.ranking);
            let filtered = intermediate.into_iter().filter(|result| result.ranking.unwrap() <= advancement_amount).collect::<Vec<_>>();
            if filtered.len() > advancement_amount {
                let not_included = filtered.last().unwrap().ranking.unwrap();
                return filtered.iter().filter(|result| result.ranking.unwrap() != not_included).map(|result| result.person_id).collect();
            }
            filtered.iter().map(|result| result.person_id).collect()
        }
    }
}
