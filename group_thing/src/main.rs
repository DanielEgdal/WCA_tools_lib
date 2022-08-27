use std::{collections::HashMap, io::Cursor};

use wca_oauth::DateTime;

mod state;

fn main() {
    let wcif = wca_oauth::parse(std::fs::read_to_string("wcif.json").unwrap()).unwrap();
    let state = state::State::new(wcif, &mut Cursor::new("man 333mbf 1..2 scram 0 01:07:10 01:07:45;"));
    //println!("{:?}", state);


    /*let overlaps = overlap::find_overlaps(&acts);
    let combination = overlap::get_group_combinations(&acts, &overlaps[1]);
    acts.print_idx();
    println!("{:?}", combination);
    println!("{:?}", overlaps);*/
}