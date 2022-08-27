use printpdf::{Mm, Point, Line, PdfLayerReference};
use std::collections::HashMap;
use crate::language::Language;
use crate::font::{FontWidth, FontPDF};
use crate::{Scorecard, TimeLimit};
use self::Alignment::*;

pub fn draw_scorecard(number: i8, Scorecard { id, round, group, station, event, stage }: &Scorecard, competition: &str, current_layer: &PdfLayerReference, font: &FontPDF, font2: &FontWidth,  font_bold: &FontPDF, font2_bold: &FontWidth, map: &HashMap<usize, String>, limits: &HashMap<&str, TimeLimit>, language: &Language) {
    let (write_text, draw_square) = get_funcs(number, font2, current_layer, font);
    let (write_bold_text, _) = get_funcs(number, font2_bold, current_layer, font_bold);
    let get_event = get_event_func(language);
    //Competiton
    write_text(competition, Centered, 52.5, 7.0, 10.0);
    let (round_text, event_text, group_text) = (format!("{}: {} | ", language.round, round), format!("{}", get_event(event)), format!(" | {}: {}", language.group, group));
    let (round_width, event_width, group_width) = (get_width_of_string(font2, &round_text, 10.0), get_width_of_string(font2_bold, &event_text, 10.0), get_width_of_string(font2, &group_text, 10.0));
    write_text(&round_text, Left, 52.5 - (round_width + event_width + group_width) / 2.0, 11.5, 10.0);
    write_bold_text(&event_text, Left, 52.5 - (- round_width + event_width + group_width) / 2.0, 11.5, 10.0);
    write_text(&group_text, Left, 52.5 - (- round_width - event_width + group_width) / 2.0, 11.5, 10.0);
    draw_square(5.0, 15.0, 10.0, 5.5);
    write_text(id.to_string().as_str(), Centered, 10.0, 19.0, 10.0);
    draw_square(15.0, 15.0, 85.0, 5.5);
    write_text(&map[id], Left, 16.0, 19.0, 10.0);

    let attempts_amount = match *event {
        "666" | "777" | "333mbf" | "333bf" | "444bf" | "555bf" => 3,
        _ => 5
    };

    let height = 8.2;
    let distance = 8.8;
    let sign_box_width = 10.0;
    let mut attempts_start_height = 25.5;
    write_text(&language.scram, Centered, 9.0 + sign_box_width / 2.0, attempts_start_height - 1.0, 7.0);
    write_text(&language.result, Centered, (12.0 + 97.0 - sign_box_width) / 2.0, attempts_start_height - 1.0, 7.0);
    write_text(&language.judge, Centered, 100.0 - sign_box_width - (sign_box_width / 2.0), attempts_start_height - 1.0, 7.0);
    write_text(&language.comp, Centered, 100.0 - (sign_box_width / 2.0), attempts_start_height - 1.0, 7.0);
    for i in 0..attempts_amount {
        let j = i as f64;
        draw_square(9.0, attempts_start_height + j * distance, sign_box_width, height);
        write_text((i + 1).to_string().as_str(), Left, 5.0, attempts_start_height - 2.0 + j * distance + height, 12.0);
        draw_square(9.0 + sign_box_width, attempts_start_height + j * distance, 91.0 - 3.0 * sign_box_width, height);
        draw_square(100.0 - 2.0 * sign_box_width, attempts_start_height + j * distance, sign_box_width, height);
        draw_square(100.0 - sign_box_width, attempts_start_height + j * distance, sign_box_width, height);
    }

    attempts_start_height += attempts_amount as f64 * distance + 3.8;
    write_text(&language.extra_attempts, Centered, 52.5, attempts_start_height - 1.0, 7.0);
    for i in 0..2 {
        let j = i as f64;
        draw_square(9.0, attempts_start_height + j * distance, sign_box_width, height);
        write_text("_", Left, 5.0, attempts_start_height - 2.0 + j * distance + height, 12.0);
        draw_square(9.0 + sign_box_width, attempts_start_height + j * distance, 91.0 - 3.0 * sign_box_width, height);
        draw_square(100.0 - 2.0 * sign_box_width, attempts_start_height + j * distance, sign_box_width, height);
        draw_square(100.0 - sign_box_width, attempts_start_height + j * distance, sign_box_width, height);
    }

    let limit = match &limits[event.clone()] {
        TimeLimit::Single(z) => format!("{}: {}", language.time_limit, time_string(*z)),
        TimeLimit::Cumulative(z) => format!("{}: {}", language.cumulative_limit, time_string(*z)),
        TimeLimit::Cutoff(x, z) => format!("{}: {}, {}: {}", language.curoff, time_string(*x), language.time_limit, time_string(*z)),
        TimeLimit::SharedCumulative(z, vec) => format!("{}: {} {} {}", language.cumulative_limit, time_string(*z), language.for_scl, vec.iter().map(|x|get_event(x)).collect::<Vec<_>>().join(&format!(" {} ", language.and_scl))),
        TimeLimit::Multi => language.multi_tl.to_owned(),
        TimeLimit::None => format!("")
    };

    if get_width_of_string(&font2, &limit, 7.0) <= 95.0 {
        write_text(&limit, Right, 100.0, 94.0, 7.0);
    }
    let station_text = format!("{}{}",
        match stage {
            Some(_) => "".to_string(), //Removed the stage to be used. I know this is ridiculous
            None => "".to_string()
        },
        match station {
            Some(v) => v.to_string(),
            None => "".to_string()
        });
    write_bold_text(&station_text, Right, 100.0, 12.0, 20.0);
}

fn time_string(mut z: usize) -> String {
    if z >= 6000 {
        let minutes = z / 6000;
        let res = format!("{}:", minutes);
        z = z % 6000;
        format!("{}{:02}.{:02}", res, z / 100, z % 100)
    } else {
        format!("{}.{:02}", z / 100, z % 100)
    }
}

enum Alignment {
    Left,
    Centered,
    Right
}

fn get_funcs<'a>(number: i8, font_path: &'a FontWidth, current_layer: &'a PdfLayerReference, font: &'a FontPDF) -> (
    Box<dyn 'a + Fn(&str, Alignment, f64, f64, f64)>,
    Box<dyn 'a + Fn(f64, f64, f64, f64)>) {
    let (x, y) = match number {
        0 => (0.0, 297.0),
        1 => (105.0, 297.0),
        2 => (0.0, 198.0),
        3 => (105.0, 198.0),
        4 => (0.0, 99.0),
        5 => (105.0, 99.0),
        _ => unreachable!()
    };
    (Box::new(move |text, alignment, x1, y1, font_size|{
        current_layer.begin_text_section();
            current_layer.set_font(font, font_size);
            current_layer.set_text_cursor(Mm(match alignment {
                Left => x + x1,
                Centered => x + x1 - (get_width_of_string(font_path ,text, font_size) / 2.0),
                Right => x + x1 - get_width_of_string(font_path ,text, font_size)
            }), Mm(y - y1));
            current_layer.set_line_height(12.0);
            current_layer.write_text(text, font);
        current_layer.end_text_section();
    }),
    Box::new(move |x1, y1, width, height|{
        let points = vec![(Point::new(Mm(x + x1), Mm(y - y1)), false),
        (Point::new(Mm(x + x1 + width), Mm(y - y1)), false),
        (Point::new(Mm(x + x1 + width), Mm(y - y1 - height)), false),
        (Point::new(Mm(x + x1), Mm(y - y1 - height)), false)];
        let square = Line {
            points,
            is_closed: true,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        current_layer.add_shape(square);
    }))
}

fn get_event_func<'a>(language: &'a Language) -> Box<dyn 'a + Fn(&str) -> &'a str> {
    Box::new(move |x|match x {
        "333" => &language.e333,
        "444" => &language.e444,
        "555" => &language.e555,
        "666" => &language.e666,
        "777" => &language.e777,
        "222" => &language.e222,
        "333oh" => &language.e333oh,
        "333fm" => "Filter out FMC",
        "333bf" => &language.e333bf,
        "pyram" => &language.epyram,
        "333mbf" => &language.e333mbf,
        "minx" => &language.eminx,
        "clock" => &language.eclock,
        "444bf" => &language.e444bf,
        "555bf" => &language.e555bf,
        "skewb" => &language.eskewb,
        "sq1" => &language.esq1,
        _ => "Please fix your csv"
    })
}

pub fn get_width_of_string(font: &FontWidth, string: &str, font_size: f64) -> f64 {
    let upem = font.metrics().units_per_em;
    let mut width = 0.0;
    for char in string.chars() {
        if !char.is_whitespace() {
            let id = font.glyph_for_char(char).unwrap();
            let glyph_width = font.advance(id).unwrap().x();
            width += glyph_width

        } else {
            width += upem as f32 / 4.0;
        }
    }
    (width as f64 / (upem as f64 / font_size)) / 2.83
}
