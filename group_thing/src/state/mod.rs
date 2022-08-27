use std::{collections::HashMap, io::Read, ops::Range};

use wca_oauth::{WcifContainer, Person, AttemptResult, DateTime};

use self::{settings::Settings, event::Event, assign_item::{AssignItem, Grouped, PreAssignment}, stage::Stage};

pub mod event;
pub mod settings;
pub mod assign_item;
pub mod stage;

pub struct State {
    cont: WcifContainer,
    settings: Settings,
    person_map: Vec<Option<usize>>,
    person_cost_map: Vec<Option<PersonCost>>,
    assign_items: Vec<AssignItem>,
    stages: Vec<Stage>,
    event_times: HashMap<Event, (DateTime, DateTime)>,
    event_attempt_times: HashMap<(Event, Range<usize>), (DateTime, DateTime)>,
    person_times: HashMap<usize, (DateTime, DateTime)>
}

struct PersonCost {
    debt: f32
}

impl State {
    pub fn new(cont: WcifContainer, settings: &mut dyn Read) -> State {
        let person_map = person_map(&cont);
        let mut state = State { cont, 
            settings: Settings::new(settings),
            person_map,
            person_cost_map: vec![],
            assign_items: vec![],
            stages: vec![],
            event_times: HashMap::new(),
            event_attempt_times: HashMap::new(),
            person_times: HashMap::new()
        };
        state.person_cost_map();
        state.stages();
        state.assign_items(vec![Grouped { events: vec![(Event::new("666").unwrap(), 0..5), (Event::new("777").unwrap(), 0..5)] } ]);
        println!("{:#?}", state.assign_items);
        state
    }

    pub fn get_person(&self, id: usize) -> Option<&Person> {
        Some(&self.cont.get().persons[self.person_map[id - 1]?])
    }

    pub fn get_pb(&self, id: usize, event: Event) -> Option<&AttemptResult> {
        self.get_person(id)?.personal_bests.iter().find(|pb| &pb.event_id == event.id() && &pb.t == event.main_type()).map(|pb| &pb.best)
    }

    fn person_cost_map(&mut self) {
        self.person_cost_map = (0..self.person_map.len()).map(|idx|{
                let person = self.get_person(idx + 1);
                person.map(|person| {
                    let debt = person.registration.as_ref().unwrap().event_ids.iter().map(|event| self.settings.competing_cost(&Event::new(event).unwrap())).sum();
                    PersonCost { debt }
                })
            })
            .collect();
    }

    fn stages(&mut self) {
        self.stages = self.cont.get().schedule.venues.iter().map(|venue| &venue.rooms).flatten().map(|_| Stage::Single { size: 30 }).collect(); //Change constant 30 to something else
    }

    fn assign_items(&mut self, groups: Vec<Grouped>) {
        let pre_assignments: Vec<_> = self.cont.get().schedule.venues.iter()
            .map(|venue| &venue.rooms)
            .flatten()
            .enumerate()
            .map(|(stage, room)| {
                let events: Vec<_> = room.activities.iter()
                    .filter(|act|{
                        act.activity_code.contains("-r1")
                    })
                    .map(|act|{
                        let event = Event::new(act.activity_code.split("-").next().unwrap()).unwrap();
                        let attempts = act.activity_code.split("-").nth(2).map(|x|{
                            let x: usize = x[1..].parse().unwrap();
                            x - 1..x
                        }).unwrap_or_else(||0..5);
                        match self.event_times.get_mut(&event) {
                            None => {
                                self.event_times.insert(event.clone(), (act.start_time, act.end_time));
                            }
                            Some(v @ &mut (start, end)) => {
                                *v = (start.min(act.start_time), end.max(act.end_time));
                            }
                        }
                        self.event_attempt_times.insert((event.clone(), attempts.clone()), (act.start_time, act.end_time));
                        (event, attempts)
                    })
                    .collect();
                
                //Dealing with grouped events...
                groups.iter()
                    .filter(|group| group.events.iter().all(|event| events.contains(&event)))
                    .map(|group| {
                        let times: Vec<_> = group.events.iter().map(|event| self.event_times[&event.0]).collect();
                        for i in 1..times.len() {
                            assert_eq!(times[i - 1], times[i]);
                        }
                        let no_of_competitors = self.cont.persons_iter().filter(|p| if let Some(r) = &p.registration {
                            r.status == "accepted" && group.events.iter().any(|event| r.event_ids.contains(&event.0.id().to_string()))
                        } else { false }).count();
                        PreAssignment::new(group.events.clone(), stage, self.stages[stage].size(), times[0].0, times[0].1, no_of_competitors)
                    })
                    //Dealing with single events ...
                    .chain(events.iter()
                        .filter(|event| groups.iter().all(|g|!g.events.contains(event)))
                        .map(|event| {
                            let no_of_competitors = self.cont.persons_iter().filter(|p| if let Some(r) = &p.registration {
                                r.status == "accepted" && r.event_ids.contains(&event.0.id().to_string())
                            } else { false }).count();
                            PreAssignment::new(vec![event.clone()], stage, self.stages[stage].size(), self.event_attempt_times[event].0, self.event_attempt_times[event].1, no_of_competitors)
                        })
                    )
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        let no_of_activities = pre_assignments.iter().map(|p|p.number_of_activities(self)).sum();
        self.assign_items = pre_assignments.iter().fold(vec![], |acc, next| next.extend(acc, self, no_of_activities));
    }

    pub fn get_debt(&self, id: usize) -> Option<f32> {
        self.person_cost_map[id - 1].as_ref().map(|opt|opt.debt)
    }
}

fn person_map(cont: &WcifContainer) -> Vec<Option<usize>> {
    cont.persons_iter().enumerate().filter(|(_, p)| p.registrant_id.is_some()).map(|(idx, p)| if &p.registration.as_ref().unwrap().status == "accepted" { Some(idx) } else { None }).collect()
}