use std::ops::Range;

use fixedbitset::FixedBitSet;
use wca_oauth::{DateTime, Role, Date};

use super::{event::Event, State};

pub struct Grouped {
    pub events: Vec<(Event, Range<usize>)>
}

#[derive(Debug)]
pub struct PreAssignment {
    events: Vec<(Event, Range<usize>)>,
    group_sizes: Vec<usize>,
    stage: usize,
    start: DateTime,
    end: DateTime
}

impl PreAssignment {
    pub fn new(events: Vec<(Event, Range<usize>)>, stage: usize, stage_size: usize, start: DateTime, end: DateTime, no_of_competitors: usize) -> PreAssignment {
        let no_of_groups = (no_of_competitors + 1) / stage_size + 1; //Integer division rounded up
        let group_size = no_of_competitors / no_of_groups;
        let leftover = no_of_competitors - group_size * no_of_groups;
        let group_sizes = (0..no_of_groups).map(|i| if i < leftover { group_size + 1 } else { group_size }).collect();
        PreAssignment { events, group_sizes, stage, start, end }
    }

    pub fn number_of_activities(&self, state: &State) -> usize {
        self.group_sizes.len() * 4
    }

    pub fn extend(&self, mut vec: Vec<AssignItem>, state: &State, no_of_activities: usize) -> Vec<AssignItem> {
        let no_of_competitors = state.cont.persons_iter().filter(|p|p.registrant_id.is_some()).count();
        let competitors = state.cont.persons_iter().filter(|p|{
                if let Some(r) = &p.registration {
                    r.status == "accepted" &&
                    self.events.iter().any(|e| r.event_ids.contains(&e.0.id().to_string()))
                }
                else {
                    false
                }
            })
            .map(|p| {
                p.registrant_id.unwrap()
            })
            .collect::<Vec<_>>();
        let delegates = competitors.iter().filter(|id|{
            state.get_person(**id).unwrap().roles.contains(&Role::Delegate)
        }).collect::<Vec<_>>();
        let idx = vec.len();
        let group_len = (self.end - self.start) / self.group_sizes.len();
        let comp: Vec<_> = (0..self.group_sizes.len()).map(|x| idx + x * 4).collect();
        for (idx, group) in self.group_sizes.iter().enumerate() {
            let start = self.start + group_len * idx;
            let end = start + group_len;
            let mut com = AssignItem::new(self.events.clone(), self.stage, AssignType::Competing, *group, no_of_competitors, &comp, start, end, no_of_activities);
            for comp in &competitors {
                com.candidates.put(*comp - 1);
            }
            vec.push(com);
            let jud = AssignItem::new(self.events.clone(), self.stage, AssignType::Judging, *group, no_of_competitors, &[], start, end, no_of_activities);
            vec.push(jud);
            let (starts, ends) = state.settings.competiting_times(&self.events[0], idx, AssignType::Scrambling, state.cont.get().schedule.start_date).unwrap_or_else(||(start, end));
            let scr = AssignItem::new(self.events.clone(), self.stage, AssignType::Scrambling, *group, no_of_competitors, &[], starts, ends, no_of_activities);
            vec.push(scr);
            let mut del = AssignItem::new(self.events.clone(), self.stage, AssignType::Delegating, *group, no_of_competitors, &[], start, end, no_of_activities);
            for dele in &delegates {
                del.candidates.put(**dele - 1);
            }
            vec.push(del);
        }
        vec
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignType {
    Competing,
    Judging,
    Scrambling,
    Delegating
}

#[derive(Debug)]
pub struct AssignItem {
    events: Vec<(Event, Range<usize>)>,
    stage: usize,
    t: AssignType,
    capacity: usize,
    assigned: FixedBitSet,
    candidates: FixedBitSet,
    conflicts: FixedBitSet,
    start: DateTime,
    end: DateTime
}

impl AssignItem {
    pub fn new(events: Vec<(Event, Range<usize>)>, stage: usize, t: AssignType, capacity: usize, no_of_competitors: usize, conflicts: &[usize], start: DateTime, end: DateTime, no_of_activities: usize) -> AssignItem {
        let mut ret = AssignItem { events, stage, t, capacity, assigned: FixedBitSet::with_capacity(no_of_competitors), candidates: FixedBitSet::with_capacity(no_of_competitors), conflicts: FixedBitSet::with_capacity(no_of_activities), start, end };
        for c in conflicts {
            ret.conflicts.put(*c);
        }
        ret
    }
}