use printpdf::PdfDocumentReference;
use wca_oauth::{WcifOAuth, Assignment, AssignmentCode};
use std::collections::HashMap;
use std::io::Write;
use crate::language::Language;
use crate::draw_scorecards::draw_scorecard;
use crate::scorecard_generator::ScorecardGenerator;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scorecard<'a> {
    pub event: &'a str,
    pub round: usize,
    pub group: usize,
    pub station: Option<usize>,
    pub id: usize,
    pub stage: Option<&'a str>
}

pub enum TimeLimit {
    Single(usize),
    Cumulative(usize),
    SharedCumulative(usize, Vec<String>),
    Cutoff(usize, usize),
    Multi,
    None
}

pub enum Return {
    Zip(Vec<u8>),
    Pdf(Vec<u8>)
}

pub fn scorecards_to_pdf(scorecards: Vec<Scorecard>, competition: &str, map: &HashMap<usize, String>, limits: &HashMap<&str, TimeLimit>, language: Language, wcif: Option<&mut WcifOAuth>) -> Result<Return, Return> {
    let mut res = Ok(());
    if let Some(wcif) = wcif {
        res = try_update_wcif(wcif, &scorecards);
    }
    let mut buckets = HashMap::new();
    for scorecard in scorecards {
        let key = scorecard.stage;
        match buckets.get_mut(&key) {
            None => {
                buckets.insert(key, vec![scorecard]);
            }
            Some(v) => {
                v.push(scorecard);
            }
        }
    }
    let ret = if buckets.len() == 1 {
        Return::Pdf(scorecards_to_pdf_internal(buckets.into_values().next().unwrap(), competition, map, limits, &language).save_to_bytes().unwrap())
    }
    else {
        let mut buf = vec![];
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        for (key, bucket) in buckets {
            let pdf = scorecards_to_pdf_internal(bucket, competition, map, limits, &language);
            zip.start_file(match key {
                None => "Missing_stage_scorecards.pdf".to_string(),
                Some(v) => format!("{v}_scorecards.pdf")
            }, zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored)).unwrap();
            zip.write(&pdf.save_to_bytes().unwrap()).unwrap();
        }
        zip.finish().unwrap();
        drop(zip);
        Return::Zip(buf)
    };
    match res {
        Ok(_) => Ok(ret),
        Err(_) => Err(ret),
    }
}

pub fn scorecards_to_pdf_internal(scorecards: Vec<Scorecard>, competition: &str, map: &HashMap<usize, String>, limits: &HashMap<&str, TimeLimit>, language: &Language) -> PdfDocumentReference {
    let mut scorecard_generator = ScorecardGenerator::new(competition);
    let mut scorecards: Vec<Option<Scorecard>> = scorecards.into_iter().map(|scorecard|Some(scorecard)).collect();
    while scorecards.len() % 6 != 0 {
        scorecards.push(None);
    }

    let n_pages = scorecards.len() / 6;
    scorecards = (0..scorecards.len()).map(|x|{
        let page = x / 6;
        let pos = x % 6;
        scorecards[pos * n_pages + page]
    }).collect::<Vec<Option<Scorecard>>>();

    let mut scorecard_pages = vec![];
    for i in 0..n_pages {
        scorecard_pages.push(&scorecards[(i * 6)..(i * 6) + 6])
    }

    for (page, scorecards) in scorecard_pages.into_iter().enumerate() {
        scorecard_generator.set_page(page);
        for (position, scorecard) in scorecards.into_iter().enumerate() {
            scorecard_generator.set_position(position);
            if let Some(scorecard) = scorecard {
                draw_scorecard(&mut scorecard_generator, scorecard, map, limits, &language);
            }
        }
    }
    scorecard_generator.doc()
}

fn try_update_wcif(wcif: &mut WcifOAuth, scorecards: &[Scorecard]) -> Result<(), String> {
    let mut event_map = HashMap::new();
        for scorecard in scorecards.iter() {
            let event = match event_map.get_mut(&(scorecard.event, scorecard.round)) {
                Some(v) => {
                    v
                }
                None => {
                    event_map.insert((scorecard.event, scorecard.round), vec![]);
                    event_map.get_mut(&(scorecard.event, scorecard.round)).unwrap()
                }
            };
            while event.len() < scorecard.group {
                event.push(vec![]);
            }
            event[scorecard.group - 1].push((scorecard.id, scorecard.station));
        }
        for ((event, round), groups) in event_map {
            let activities = wcif.add_groups_to_event(event, round, groups.len()).map_err(|_|format!("Unable to add groups to event: {event}. This may be because groups are already created or the event does not exist."))?;
            let acts = activities.iter().map(|act|act.id).collect::<Vec<_>>();
            for (group, activity_id) in groups.into_iter().zip(acts) {
                for (id, station) in group {
                    wcif.patch_persons(|person|{
                        if person.registrant_id == Some(id) {
                            person.assignments.push(Assignment { activity_id, assignment_code: AssignmentCode::Competitor, station_number: station })
                        }
                    })

                }
            }
        }
    Ok(())
}

