use fixedbitset::FixedBitSet;
use wca_oauth::{DateTime, WcifContainer, Time};

use crate::{event::{ActivityIdentifier, Event}, settings::Settings, competitor::Competitor};

#[derive(Debug)]
pub struct PreActivity {
    stage: usize,
    pub events: Vec<ActivityIdentifier>,
    candidates: FixedBitSet,
    pub start: DateTime,
    pub end: DateTime
}

impl PreActivity {
    pub fn pre_activities(wcif: &WcifContainer, competitors: &mut [Option<Competitor>], settings: &Settings) -> Vec<PreActivity> {
        let mut used_shared = vec![];
        wcif.get().schedule.venues.iter().flat_map(|venue| {
                &venue.rooms
            })
            .enumerate()
            .flat_map(|(idx, room)| {
                std::iter::repeat(idx).zip(&room.activities)
            })
            .filter_map(|(idx, activity)| {
                if activity.activity_code.contains("other") {
                    return None;
                }
                let mut iter = activity.activity_code.split("-");
                let event_id = iter.next().unwrap();
                let round: usize = iter.next().unwrap()[1..].parse().unwrap();
                if round != 1 {
                    return None;
                }
                let attempt: Option<usize> = iter.next().map(|s|s[1..].parse().unwrap());
                let event = wcif.events_iter().find(|event| event.id == event_id).unwrap();
                let events = if let Some(time_limit) = &event.rounds[0].time_limit {
                    if used_shared.contains(&time_limit.cumulative_round_ids[0]) {
                        return None;
                    }
                    else {
                        for event in time_limit.cumulative_round_ids.iter() {
                            used_shared.push(event.to_string());
                        }
                    }
                    time_limit.cumulative_round_ids.iter().map(|id| ActivityIdentifier::new(Event::new(id.split("-").next().unwrap()).unwrap(), attempt)).collect()
                }
                else {
                    vec![ActivityIdentifier::new(Event::new(event_id).unwrap(), attempt)]
                };
                let dur = activity.start_time.signed_duration_since(wcif.date().and_time(Time::from_hms(0, 0, 0)));
                let day = dur.num_days() as usize;
                let time = (activity.start_time, activity.end_time);
                let candidates = wcif.persons_iter()
                    .filter_map(|p| {
                        let reg = p.registration.as_ref()?;
                        if reg.status == "accepted" && events.iter().any(|event| reg.event_ids.contains(&event.event.id().to_string())) {
                            if let Some(comp) = &mut competitors[p.registrant_id.unwrap() - 1] {
                                for event in events.iter() {
                                    comp.add_debt(settings.competing_cost(&event));
                                    comp.add_availability(day, time);
                                }
                            }
                            p.registrant_id.map(|x|x - 1)
                        }
                        else {
                            None
                        }
                    })
                    .collect();
                Some(PreActivity {
                    stage: idx, 
                    events,
                    candidates,
                    start: activity.start_time,
                    end: activity.end_time
                })
            }).collect()
    }

    pub fn into_activities(mut self, settings: &Settings) -> Vec<Activity> {
        let id_no = settings.id_no();
        self.candidates.grow(id_no);
        let no_of_competitors = self.candidates.ones().count();
        let stage_size = settings.stage_size(self.stage);
        let no_of_groups = ((no_of_competitors - 1) / stage_size + 1).max(settings.min_groups(&self.events[0]));
        let group_size = no_of_competitors / no_of_groups;
        let leftovers = no_of_competitors - group_size * no_of_groups;
        let group_sizes: Vec<_> = (0..no_of_groups).map(|g| {
            group_size + if g < leftovers { 1 } else { 0 }
        }).collect();
        let mut acts = vec![];
        let group_time = self.end.signed_duration_since(self.start) / no_of_groups as i32;
        for (idx, group_size) in group_sizes.into_iter().enumerate() {
            let start = self.start + group_time * idx as i32;
            let end = start + group_time;
            acts.push(Activity::new(self.events.clone(), idx, self.candidates.clone(), group_size, ActivityType::Competiter, start, end, id_no));
            if !settings.no_scram(&self.events[0]) {
                let sc: f32 = self.events.iter().map(|e| settings.scramble_cost(&e.event)).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                acts.push(Activity::new(self.events.clone(), idx, self.candidates.clone(), (group_size as f32 * sc).ceil() as usize, ActivityType::Scrambler, start, end, id_no));
            }
            if !settings.no_judge(&self.events[0]) {
                let jc: f32 = self.events.iter().map(|e| settings.judge_cost(&e.event)).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                acts.push(Activity::new(self.events.clone(), idx, self.candidates.clone(), (group_size as f32 * jc).ceil() as usize, ActivityType::Judge, start, end, id_no));
            }
        }
        acts
    }
}

#[derive(Debug)]
pub struct Activity {
    pub id: Vec<ActivityIdentifier>,
    pub group: usize,
    pub t: ActivityType,
    pub candidates: FixedBitSet,
    pub preferred_candidates: FixedBitSet,
    pub assigned: FixedBitSet,
    pub capacity: usize,
    pub start: DateTime,
    pub end: DateTime,
    pub delegates_assinged: usize
}

impl Activity {
    pub fn new(id: Vec<ActivityIdentifier>, group: usize, competitors: FixedBitSet, capacity: usize, t: ActivityType, start: DateTime, end: DateTime, id_no: usize) -> Activity {
        Activity { id, group, t, candidates: FixedBitSet::with_capacity(id_no), preferred_candidates: competitors, assigned: FixedBitSet::with_capacity(id_no), capacity, start, end, delegates_assinged: 0 }
    }

    pub fn collides(&self, other: &Activity, _settings: &Settings) -> bool {
        self.start < other.end && other.start < self.end ||
        self.id == other.id && (self.group == other.group || (self.t == ActivityType::Competiter && other.t == ActivityType::Competiter))
    }

    pub fn remove_candidate(&mut self, id: usize) {
        self.candidates.set(id, false);
        self.preferred_candidates.set(id, false);
    }

    pub fn assign(&mut self, id: usize) {
        self.assigned.set(id, true);
        self.capacity -= 1;
        self.remove_candidate(id)
    }

    pub fn assign_delegate(&mut self, id: usize) {
        self.delegates_assinged += 1;
        self.assign(id);
    }

    pub fn available_leftover(&self) -> f32 {
        self.candidates.count_ones(..) as f32 / self.capacity as f32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ActivityType {
    Competiter,
    Judge,
    Scrambler
}