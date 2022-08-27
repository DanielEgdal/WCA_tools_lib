use printpdf::{PdfDocumentReference, PdfDocument, Mm, Point, Line, LineDashPattern, Color, Greyscale};
use wca_oauth::{WcifOAuth, Assignment, AssignmentCode};
use std::collections::HashMap;
use std::io::Write;
use crate::language::Language;
use crate::font::load_fonts;
use crate::draw_scorecards::draw_scorecard;

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

pub fn scorecards_to_pdf(scorecards: Vec<Scorecard>, competition: &str, map: &HashMap<usize, String>, limits: &HashMap<&str, TimeLimit>, language: Language, wcif: Option<&mut WcifOAuth>) -> Return {
    if let Some(wcif) = wcif {
            try_update_wcif(wcif, &scorecards).unwrap_or_else(|e| println!("{}", e));
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
    let (doc, page, layer) = PdfDocument::new(competition, Mm(210.0), Mm(297.0), "Layer 1");
    let mut pages = vec![(page, layer)];
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
    for _ in 1..scorecard_pages.len() {
        let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
        pages.push((page, layer));
    }
    let pages = pages.into_iter().zip(scorecard_pages);

    let (font_width, font) = load_fonts(&doc, "normal");
    let (font_width_bold, font_bold) = load_fonts(&doc, "bold");
    for ((page, layer), scorecards) in pages {
        let current_layer = doc.get_page(page).get_layer(layer);
        let points1 = vec![(Point::new(Mm(105.0), Mm(0.0)), false),
                        (Point::new(Mm(105.0), Mm(297.0)), false)];
        let points2 = vec![(Point::new(Mm(0.0), Mm(99.0)), false),
                        (Point::new(Mm(210.0), Mm(99.0)), false)];
        let points3 = vec![(Point::new(Mm(0.0), Mm(198.0)), false),
                        (Point::new(Mm(210.0), Mm(198.0)), false)];
        let line1 = line_from_points(points1);
        let line2 = line_from_points(points2);
        let line3 = line_from_points(points3);
        let width = Some(5);
        let gap = Some(10);
        let dash_pattern = LineDashPattern::new(0, width, gap, width, gap, width, gap);
        let outline_color = Color::Greyscale(Greyscale::new(0.0, None));
        current_layer.set_overprint_stroke(true);
        current_layer.set_line_dash_pattern(dash_pattern);
        current_layer.set_outline_color(outline_color);
        current_layer.set_outline_thickness(0.5);
        current_layer.add_shape(line1);
        current_layer.add_shape(line2);
        current_layer.add_shape(line3);
        
        let dash_pattern = LineDashPattern::new(0, None, None, None, None, None, None);
        current_layer.set_line_dash_pattern(dash_pattern);

        for (scorecard, number) in scorecards.into_iter().zip(0..6) {
            match scorecard {
                None => (),
                Some(v) => draw_scorecard(number, v, competition, &current_layer, &font, &font_width, &font_bold, &font_width_bold, map, limits, &language)
            }
        }
    }
    doc
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

fn line_from_points(points: Vec<(Point, bool)>) -> Line {
    Line {
        points,
        is_closed: false,
        has_fill: false,
        has_stroke: true,
        is_clipping_path: false,
    }
}

