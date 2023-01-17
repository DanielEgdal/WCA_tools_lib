use printpdf::PdfDocumentReference;
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
    pub stage: Option<u32>
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

pub fn scorecards_to_pdf(scorecards: Vec<Scorecard>, competition: &str, map: &HashMap<usize, String>, limits: &HashMap<&str, TimeLimit>, language: Language) -> Return {
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
    if buckets.len() == 1 {
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