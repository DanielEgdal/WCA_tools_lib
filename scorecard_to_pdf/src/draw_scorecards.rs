use std::collections::HashMap;
use crate::language::Language;
use crate::scorecard::MaybeScorecard;
use crate::scorecard_generator::ScorecardGenerator;
use crate::TimeLimit;
use crate::scorecard_generator::{Alignment::*, Weight::*};

pub fn draw_scorecard(generator: &mut ScorecardGenerator, scorecard: &MaybeScorecard, map: &HashMap<usize, String>, limits: &HashMap<&str, TimeLimit>, language: &Language) {
    let get_event = get_event_func(language);
    //Competiton
    let name = generator.get_competition_name().to_string();
    generator.write(&name, 52.5, 7.0, 10.0, Center, Normal);
    let round_text = format!("{}: {} | ", language.round, scorecard.round());
    let event_text = format!("{}", get_event(scorecard.event()));
    let group_text = format!(" | {}: {}", language.group, scorecard.group());
    generator.write_multi_text(52.5, 11.5, 10.0, Center, &[
        (&round_text, Normal),
        (&event_text, Bold),
        (&group_text, Normal),
    ]);
    generator.draw_square(5.0, 15.0, 10.0, 5.5);
    generator.write(&scorecard.id(), 10.0, 19.0, 10.0, Center, Normal);
    generator.draw_square(15.0, 15.0, 85.0, 5.5);
    generator.write(scorecard.name(map), 16.0, 19.0, 10.0, Left, Normal);

    let attempts_amount = match scorecard.event() {
        "666" | "777" | "333mbf" | "333bf" | "444bf" | "555bf" => 3,
        _ => 5
    };

    let height = 8.2;
    let distance = 8.8;
    let sign_box_width = 10.0;
    let mut attempts_start_height = 25.5;
    generator.write(&language.scram, 9.0 + sign_box_width / 2.0, attempts_start_height - 1.0, 7.0, Center, Normal);
    generator.write(&language.result, (12.0 + 97.0 - sign_box_width) / 2.0, attempts_start_height - 1.0, 7.0, Center, Normal);
    generator.write(&language.judge, 100.0 - sign_box_width - (sign_box_width / 2.0), attempts_start_height - 1.0, 7.0, Center, Normal);
    generator.write(&language.comp, 100.0 - (sign_box_width / 2.0), attempts_start_height - 1.0, 7.0, Center, Normal);
    for i in 0..attempts_amount {
        let j = i as f64;
        generator.draw_square(9.0, attempts_start_height + j * distance, sign_box_width, height);
        generator.write((i + 1).to_string().as_str(), 5.0, attempts_start_height - 2.0 + j * distance + height, 12.0, Left, Normal);
        generator.draw_square(9.0 + sign_box_width, attempts_start_height + j * distance, 91.0 - 3.0 * sign_box_width, height);
        generator.draw_square(100.0 - 2.0 * sign_box_width, attempts_start_height + j * distance, sign_box_width, height);
        generator.draw_square(100.0 - sign_box_width, attempts_start_height + j * distance, sign_box_width, height);
    }

    attempts_start_height += attempts_amount as f64 * distance + 3.8;
    generator.write(&language.extra_attempts, 52.5, attempts_start_height - 1.0, 7.0, Center, Normal);
    for i in 0..2 {
        let j = i as f64;
        generator.draw_square(9.0, attempts_start_height + j * distance, sign_box_width, height);
        generator.write("_", 5.0, attempts_start_height - 2.0 + j * distance + height, 12.0, Left, Normal);
        generator.draw_square(9.0 + sign_box_width, attempts_start_height + j * distance, 91.0 - 3.0 * sign_box_width, height);
        generator.draw_square(100.0 - 2.0 * sign_box_width, attempts_start_height + j * distance, sign_box_width, height);
        generator.draw_square(100.0 - sign_box_width, attempts_start_height + j * distance, sign_box_width, height);
    }

    let limit = match scorecard.limit(limits) {
        TimeLimit::Single(z) => format!("{}: {}", language.time_limit, time_string(*z)),
        TimeLimit::Cumulative(z) => format!("{}: {}", language.cumulative_limit, time_string(*z)),
        TimeLimit::Cutoff(x, z) => format!("{}: {}, {}: {}", language.curoff, time_string(*x), language.time_limit, time_string(*z)),
        TimeLimit::SharedCumulative(z, vec) => format!("{}: {} {} {}", language.cumulative_limit, time_string(*z), language.for_scl, vec.iter().map(|x|get_event(x)).collect::<Vec<_>>().join(&format!(" {} ", language.and_scl))),
        TimeLimit::Multi => language.multi_tl.to_owned(),
        TimeLimit::None => format!("")
    };

    if generator.get_width_of_string(&limit, 7.0, Normal) <= 95.0 {
        generator.write(&limit, 100.0, 94.0, 7.0, Right, Normal);
    }
    let station_text = match scorecard.station() {
        Some(v) => v.to_string(),
        None => "".to_string()
    };
    generator.write(&station_text, 100.0, 12.0, 20.0, Right, Bold);
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

fn get_event_func<'a>(language: &'a Language) -> impl Fn(&str) -> &'a str {
    |x| match x {
        "" => "___________________________",
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
    }
}