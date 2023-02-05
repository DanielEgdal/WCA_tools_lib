#![feature(future_join)]

use pdf::{run, save_pdf};
use scorecard_to_pdf::{Language, Scorecard};

mod pdf;
pub(crate) mod wcif;
mod localhost;
mod compiled;

pub use pdf::Stages;

#[allow(deprecated)]
#[deprecated]
pub fn print_round_1<I>(args: &mut I) where I: Iterator<Item = String> {
    print_round_1_with_language(args, Language::english());
}

#[deprecated]
pub fn print_round_1_with_language<I>(args: &mut I, language: Language) where I: Iterator<Item = String> {
    let a = args.next().unwrap();
    let a = std::fs::read_to_string(a).unwrap();
    let b = args.next().unwrap();
    let b = std::fs::read_to_string(b).unwrap();
    let c = args.next().unwrap();
    run(&a, Some(b), &c, language, Stages::new(1, u32::MAX), ScorecardOrdering::Default);
}

#[allow(unused)]
pub fn print_subsequent_rounds(competition_id: String, stages: Stages, sort_by_name: bool) {
    localhost::init(stages, ScorecardOrdering::from_bool(sort_by_name));
}

pub fn print_round_1_english(groups_csv: &str, limit_csv: Option<String>, competition: &str, stages: Stages, sort_by_name: bool) {
    let groups_csv = std::fs::read_to_string(groups_csv).unwrap();
    let limit_csv = limit_csv.map(|x| std::fs::read_to_string(x).unwrap());
    let compare = ScorecardOrdering::from_bool(sort_by_name);
    let scorecards = run(&groups_csv, limit_csv, competition, Language::english(), stages, compare);
    save_pdf(scorecards, competition, "").unwrap();
}

pub fn blank_scorecard_page(competition: &str) {
    save_pdf(scorecard_to_pdf::blank_scorecard_page(competition, &Language::english()), competition, "blank_").unwrap();
}

#[derive(Clone, Copy)]
pub(crate) enum ScorecardOrdering {
    Default,
    ByName,
}

impl ScorecardOrdering {
    fn from_bool(sort_by_name: bool) -> ScorecardOrdering {
        if sort_by_name {
            ScorecardOrdering::ByName
        }
        else {
            ScorecardOrdering::Default
        }
    }

    fn sort_slice(&self, slice: &mut [Scorecard<'_>]) {
        match self {
            ScorecardOrdering::Default => slice.sort(),
            ScorecardOrdering::ByName => slice.sort_by(|a, b| a.id.cmp(&b.id).then(a.cmp(&b))),
        }
    }
}
