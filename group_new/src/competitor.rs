use std::collections::HashMap;

use fixedbitset::FixedBitSet;
use wca_oauth::{Role, Person, Date, DateTime};

use crate::event::Event;

#[derive(Debug)]
pub struct Competitor {
    pub name: String,
    pub age: usize,
    pub pbs: HashMap<Event, usize>,
    pub roles: Vec<Role>,
    pub debt: f32,
    availability: Vec<Option<(DateTime, DateTime)>>,
    pub assignemtns: FixedBitSet
}

impl Competitor {
    pub fn new(person: &Person, competition_date: Date) -> Option<Competitor> {
        let reg = person.registration.as_ref()?;
        if reg.status != "accepted" {
            return None;
        }
        let age = (competition_date.signed_duration_since(person.birthdate).num_days() / 365) as usize;
        let pbs = person.personal_bests.iter().filter(|event| {
                event.event_id != "333ft" && event.event_id != "magic" && event.event_id != "mmagic" && event.event_id != "333mbo" && 
                event.t == Event::main_type(&Event::new(&event.event_id).unwrap())
            })
            .map(|event| {
                (Event::new(&event.event_id).unwrap(), match event.best {
                    wca_oauth::AttemptResult::Ok(v) => v,
                    _ => unreachable!()
                })
            })
            .collect();
        let roles = person.roles.clone();
        Some(Competitor { name: person.name.clone(), age, pbs, roles, debt: 0.0, availability: vec![], assignemtns: FixedBitSet::new() })
    }

    pub fn add_debt(&mut self, debt: f32) {
        if !self.roles.contains(&Role::Delegate) && !self.roles.contains(&Role::TraineeDelegate) && !self.roles.contains(&Role::Organizer) {
            self.debt += debt;
        }
    }

    pub fn add_availability(&mut self, day: usize, time: (DateTime, DateTime)) {
        while self.availability.len() <= day {
            self.availability.push(None);
        }
        self.availability[day] = match self.availability[day] {
            None => Some(time),
            Some(old_time) => Some((old_time.0.min(time.0), old_time.1.max(time.1)))
        }
    }

    pub fn pb(&self, event: &Event) -> Option<&usize> {
        self.pbs.get(event)
    }

    pub fn available(&self, start: &DateTime, end: &DateTime) -> bool {
        self.availability.iter().filter_map(|x|x.as_ref()).any(|(s, e)| s <= start && e >= end)
    }

    pub fn qualified_scrambler(&self, event: &Event) -> bool {
        //Consider whether this really is a good criteria for being qualified
        self.age >= 14 && match event.id() {
            "333"  => self.pbs.get(event).map(|v| *v < 1500).unwrap_or_else(||false),
            "222"  => self.pbs.get(event).map(|v| *v < 500).unwrap_or_else(||false),
            "444"  => self.pbs.get(event).map(|v| *v < 5000).unwrap_or_else(||false),
            "555"  => self.pbs.get(event).map(|v| *v < 10000).unwrap_or_else(||false),
            "666"  => self.pbs.get(event).map(|v| *v < 22000).unwrap_or_else(||false),
            "777"  => self.pbs.get(event).map(|v| *v < 30000).unwrap_or_else(||false),
            "333oh"  => self.pbs.get(event).map(|v| *v < 1700).unwrap_or_else(||false),
            "333bf"  => self.pbs.get(&Event::new("333").unwrap()).map(|v| *v < 1500).unwrap_or_else(||false),
            "444bf"  => self.pbs.get(&Event::new("444").unwrap()).map(|v| *v < 5000).unwrap_or_else(||false),
            "555bf"  => self.pbs.get(&Event::new("555").unwrap()).map(|v| *v < 8000).unwrap_or_else(||false),
            "333mbf"  => self.pbs.get(&Event::new("333").unwrap()).map(|v| *v < 1500).unwrap_or_else(||false),
            "skewb"  => self.pbs.get(event).map(|v| *v < 500).unwrap_or_else(||false),
            "pyram"  => self.pbs.get(event).map(|v| *v < 800).unwrap_or_else(||false),
            "minx"  => self.pbs.get(event).map(|v| *v < 8000).unwrap_or_else(||false),
            "sq1"  => self.pbs.get(event).map(|v| *v < 1500).unwrap_or_else(||false),
            "clock"  => self.pbs.get(event).map(|v| *v < 1500).unwrap_or_else(||false),
            _ => false
        }
    }

    pub fn qualified_judge(&self, event: &Event) -> bool {
        match event.id() {
            "444bf" | "555bf" | "333mbf" => {
                self.age > 12 && self.pbs.len() != 0
            }
            _ => true
        }
    }

    pub fn assign(&mut self, act: usize) {
        if self.assignemtns.len() <= act {
            self.assignemtns.grow(act + 1);
        }
        self.assignemtns.set(act, true);
    }
}