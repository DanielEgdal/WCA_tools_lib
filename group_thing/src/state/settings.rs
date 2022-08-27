use std::{io::Read, ops::Range};

use wca_oauth::{DateTime, Date};

use super::{event::Event, assign_item::AssignType};

pub struct Settings {
    scrabmle_cost: [f32; 17],
    judge_cost: [f32; 17],
    staff_multiplier: [f32; 17],
    competing_cost: [f32; 17],
    manuel_times: Vec<ManuelTimes>
}

struct ManuelTimes {
    id: (Event, Range<usize>),
    group: usize, 
    t: AssignType,
    start_time: DateTime,
    end_time: DateTime
}

impl Settings {
    const DEFAULT_SCRAMBLE_COST: [f32; 17] = [0.15, 0.15, 0.20, 0.20, 0.20, 0.20, 0.15, 0.00, 0.10, 0.07, 0.07, 0.15, 0.20, 0.20, 0.15, 0.25, 0.20];
    const DEFAULT_JUDGE_COST: [f32; 17] = [0.75, 0.75, 0.75, 0.75, 0.80, 0.80, 0.75, 0.00, 0.80, 0.40, 0.40, 0.75, 0.80, 0.75, 0.75, 0.33, 0.75];
    const STAFF_MULTIPLIER: [f32; 17] = [1.0, 0.9, 1.1, 1.3, 1.5, 1.5, 1.1, 0.0, 1.3, 1.7, 2.5, 0.9, 1.5, 1.0, 0.9, 3.0, 1.2];
    pub fn new(s: &mut dyn Read) -> Settings {
        let mut competing_cost: [f32; 17] = [0.0; 17];
        for idx in 0..17 {
            competing_cost[idx] = (Self::DEFAULT_SCRAMBLE_COST[idx] + Self::DEFAULT_JUDGE_COST[idx]) * Self::STAFF_MULTIPLIER[idx];
        }
        let mut settings = Settings { scrabmle_cost: Self::DEFAULT_SCRAMBLE_COST,
            judge_cost: Self::DEFAULT_JUDGE_COST,
            staff_multiplier: Self::STAFF_MULTIPLIER,
            competing_cost,
            manuel_times: vec![]
        };
        let mut buf = String::new();
        s.read_to_string(&mut buf);
        buf.split(";")
            .for_each(|statement| {
                let mut iter = statement.split_ascii_whitespace();
                match iter.next() {
                    None => (),
                    Some("man") => {
                        let event = Event::new(iter.next().unwrap()).unwrap();
                        let mut attempts = iter.next().unwrap().split("..");
                        let id = (event, attempts.next().unwrap().parse().unwrap()..attempts.next().unwrap().parse().unwrap());
                        let t = match iter.next() {
                            Some("scram") => AssignType::Scrambling,
                            Some("comp") => AssignType::Competing,
                            Some("judge") => AssignType::Judging,
                            Some("delegating") => AssignType::Delegating,
                            _ => panic!("Invalid AssignType")
                        };
                        let group = iter.next().unwrap().parse().unwrap();
                        let start_time = DateTime::new(iter.next().unwrap());
                        let end_time = DateTime::new(iter.next().unwrap());
                        settings.manuel_times.push(ManuelTimes { id, group, t, start_time, end_time })
                    }
                    _ => panic!("Invalid setting")
                }    
            });
        settings
    }

    pub fn changle_scramble_cost(&mut self, event: &Event, cost: f32) {
        self.scrabmle_cost[event.usize_id()] = cost;
    }

    pub fn scramble_cost(&self, event: &Event) -> f32 {
        self.scrabmle_cost[event.usize_id()]
    }

    pub fn change_judge_cost(&mut self, event: &Event, cost: f32) {
        self.judge_cost[event.usize_id()] = cost;
    }

    pub fn judge_cost(&self, event: &Event) -> f32 {
        self.judge_cost[event.usize_id()]
    }

    pub fn change_staff_multiplier(&mut self, event: &Event, mulitiplier: f32) {
        self.staff_multiplier[event.usize_id()] = mulitiplier;
    }

    pub fn staff_multiplier(&self, event: &Event) -> f32 {
        self.staff_multiplier[event.usize_id()]
    }

    pub fn competing_cost(&self, event: &Event) -> f32 {
        self.competing_cost[event.usize_id()]
    }

    pub fn competiting_times(&self, id: &(Event, Range<usize>), group: usize, t: AssignType, start_date: Date) -> Option<(DateTime, DateTime)> {
        let start_time: DateTime = start_date.into();
        self.manuel_times.iter()
            .find(|setting| &setting.id == id && setting.group == group && setting.t == t)
            .map(|setting|{
                (start_time + setting.start_time, start_time + setting.end_time)
            })
    }
}