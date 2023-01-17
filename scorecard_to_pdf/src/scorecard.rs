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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeScorecard<'a> {
    Blank,
    Normal(Scorecard<'a>)
}

impl<'a> MaybeScorecard<'a> {
    fn internal_or_default<T, F>(&'a self, f: F, default: T) -> T where F: Fn(&'a Scorecard) -> T, T: 'a {
        match self {
            MaybeScorecard::Blank => default,
            MaybeScorecard::Normal(s) => f(s),
        }
    }

    pub fn event(&self) -> &str {
        self.internal_or_default(|s| s.event, "")
    }

    pub fn round(&self) -> String {
        self.internal_or_default(|s| s.round.to_string(), "__".to_string())
    }

    pub fn group(&self) -> String {
        self.internal_or_default(|s| s.group.to_string(), "__".to_string())
    }

    pub fn station(&self) -> Option<usize> {
        self.internal_or_default(|s| s.station, None)
    }

    pub fn id(&self) -> String {
        self.internal_or_default(|s| s.id.to_string(), "".to_string())
    }

    pub fn name(&'a self, map: &'a HashMap<usize, String>) -> &'a str {
        self.internal_or_default(|s| &map[&s.id], "")
    }

    pub fn limit(&'a self, limit: &'a HashMap<&str, TimeLimit>) -> &'a TimeLimit {
        self.internal_or_default(|s| &limit.get(s.event).unwrap_or(&TimeLimit::None), &TimeLimit::None)
    }
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
    let mut scorecards: Vec<MaybeScorecard> = scorecards.into_iter().map(|scorecard|MaybeScorecard::Normal(scorecard)).collect();
    while scorecards.len() % 6 != 0 {
        scorecards.push(MaybeScorecard::Blank);
    }

    let n_pages = scorecards.len() / 6;
    scorecards = (0..scorecards.len()).map(|x|{
        let page = x / 6;
        let pos = x % 6;
        scorecards[pos * n_pages + page]
    }).collect::<Vec<MaybeScorecard>>();

    let mut scorecard_pages = vec![];
    for i in 0..n_pages {
        scorecard_pages.push(&scorecards[(i * 6)..(i * 6) + 6])
    }

    for (page, scorecards) in scorecard_pages.into_iter().enumerate() {
        scorecard_generator.set_page(page);
        for (position, scorecard) in scorecards.into_iter().enumerate() {
            scorecard_generator.set_position(position);
            draw_scorecard(&mut scorecard_generator, scorecard, map, limits, &language);
        }
    }
    scorecard_generator.doc()
}

pub fn blank_scorecard_page(competition: &str, language: &Language) -> Return {
    let mut scorecard_generator = ScorecardGenerator::new(competition);
    scorecard_generator.set_page(0);
    let map = HashMap::new();
    let limits = HashMap::new();
    for i in 0..6 {
        scorecard_generator.set_position(i);
        draw_scorecard(&mut scorecard_generator, &MaybeScorecard::Blank, &map, &limits, language)
    }
    Return::Pdf(scorecard_generator.doc().save_to_bytes().unwrap())
}