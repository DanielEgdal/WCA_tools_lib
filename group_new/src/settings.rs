use std::collections::HashSet;

use crate::event::{Event, ActivityIdentifier};

#[derive(Debug)]
pub struct Settings {
    scrabmle_cost: [f32; 17],
    judge_cost: [f32; 17],
    staff_multiplier: [f32; 17],
    competing_cost: [f32; 17],
    stages: Vec<Stage>,
    no_scram: HashSet<ActivityIdentifier>,
    no_judge: HashSet<ActivityIdentifier>,
    id_no: usize
}

#[derive(Debug)]
pub struct Stage {
    stages: Vec<(Option<String>, usize)>
}

impl Stage {
    pub fn size(&self) -> usize {
        self.stages.iter().map(|s|s.1).sum()
    }

    pub fn no_of_stages(&self) -> f32 {
        self.stages.len() as f32
    }
}

impl Settings {
    const DEFAULT_SCRAMBLE_COST: [f32; 17] = [0.15, 0.15, 0.20, 0.20, 0.20, 0.20, 0.15, 0.00, 0.10, 0.07, 0.07, 0.15, 0.20, 0.20, 0.15, 0.20, 0.20];
    const DEFAULT_JUDGE_COST: [f32; 17] = [0.75, 0.75, 0.75, 0.75, 0.65, 0.65, 0.75, 0.00, 0.80, 0.60, 0.60, 0.75, 0.75, 0.75, 0.75, 0.25, 0.75];
    const STAFF_MULTIPLIER: [f32; 17] = [1.0, 0.9, 1.1, 1.3, 1.5, 1.5, 1.1, 0.0, 1.3, 1.7, 2.5, 0.9, 1.5, 1.0, 0.9, 3.0, 1.2];
    pub fn new(gs: &str, id_no: usize) -> Settings {
        let mut competing_cost: [f32; 17] = [0.0; 17];
        for idx in 0..17 {
            competing_cost[idx] = (Self::DEFAULT_SCRAMBLE_COST[idx] + Self::DEFAULT_JUDGE_COST[idx]) * Self::STAFF_MULTIPLIER[idx];
        }
        let mut settings = Settings { scrabmle_cost: Self::DEFAULT_SCRAMBLE_COST,
            judge_cost: Self::DEFAULT_JUDGE_COST,
            staff_multiplier: Self::STAFF_MULTIPLIER,
            competing_cost,
            stages: vec![],
            no_scram: HashSet::new(),
            no_judge: HashSet::new(),
            id_no
        };
        for command in gs.split(";") {
            let mut iter = command.split_ascii_whitespace();
            match iter.next() {
                None => (),
                Some("stage") => {
                    let stages = iter.map(|obj| {
                        if obj.contains("-") {
                            let mut iter = obj.split("-");
                            let name = iter.next().unwrap();
                            let size = iter.next().unwrap().parse().unwrap();
                            (Some(name.to_string()), size)
                        }
                        else {
                            (None, obj.parse().unwrap())
                        }
                    })
                    .collect();
                    settings.stages.push(Stage { stages });
                }
                Some("no_judge") => {
                    settings.no_judge.insert(ActivityIdentifier::from_id(iter.next().unwrap()));
                }
                Some("no_scram") => {
                    settings.no_scram.insert(ActivityIdentifier::from_id(iter.next().unwrap()));
                }
                Some(v) => panic!("Invalid command {}", v)
            }
        }    
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

    pub fn competing_cost(&self, event: &ActivityIdentifier) -> f32 {
        if self.no_scram(event) || self.no_judge(event) {
            0.0
        }
        else {
            self.competing_cost[event.event.usize_id()]
        }
    }

    pub fn no_scram(&self, act: &ActivityIdentifier) -> bool {
        self.no_scram.contains(act)
    }

    pub fn no_judge(&self, act: &ActivityIdentifier) -> bool {
        self.no_judge.contains(act)
    }

    pub fn stage_size(&self, idx: usize) -> usize {
        self.stages[idx].size()
    }

    pub fn id_no(&self) -> usize {
        self.id_no
    }

    pub fn min_groups(&self, act: &ActivityIdentifier) -> usize {
        match (act.event.id(), act.attempt) {
            ("555bf", _) | ("444bf", _) => 2,
            _ => 0
        }
    }

    pub fn no_of_stages(&self, idx: usize) -> f32 {
        self.stages[idx].no_of_stages()
    }
}