mod font;
mod scorecard;
mod language;
mod draw_scorecards;
mod scorecard_generator;
pub use scorecard::{scorecards_to_pdf, Scorecard, TimeLimit, Return};
pub use language::Language;
