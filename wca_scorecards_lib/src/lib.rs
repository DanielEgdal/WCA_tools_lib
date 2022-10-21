use pdf::{run, save_pdf};
use scorecard_to_pdf::Language;

mod pdf;
pub(crate) mod wcif;
mod localhost;
mod compiled;
mod localhost_round_1;

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
    run(&a, &b, &c, language, None);
}

pub fn print_subsequent_rounds(competition_id: String, stages: Option<Stages>) {
    localhost::init(competition_id, stages);
}

pub fn print_round_1_english(groups_csv: &str, limit_csv: &str, competition: &str, stages: Option<Stages>) {
    let groups_csv = std::fs::read_to_string(groups_csv).unwrap();
    let limit_csv = std::fs::read_to_string(limit_csv).unwrap();
    save_pdf(run(&groups_csv, &limit_csv, competition, Language::english(), stages), competition).unwrap();
}

pub fn print_round_1_with_patch(groups_csv: &str, limit_csv: &str, id: &str) {
    let groups_csv = std::fs::read_to_string(groups_csv).unwrap();
    let limit_csv = std::fs::read_to_string(limit_csv).unwrap();
    localhost_round_1::init(id.to_string(), groups_csv, limit_csv);
}

#[cfg(test)]
mod test {
    use crate::Stages;

    #[test]
    fn everything() {
        let mut stages = Stages::new();
        stages.add_stage(Some("R".to_string()), 10);
        stages.add_stage(Some("G".to_string()), 10);
        stages.add_stage(Some("B".to_string()), 10);

        //crate::print_round_1_english("files/OstervangOpen2022stationNumbers.csv", "files/OstervangOpen2022timeLimits.csv", "Østervang Open 2022", Some(stages));
        crate::print_subsequent_rounds("danishchampionship2022".to_string(), Some(stages));
    }
}