use std::{collections::HashMap, ops::{BitAnd, BitOrAssign, BitAndAssign}};

use fixedbitset::FixedBitSet;
use wca_oauth::{WcifContainer, Role};

use crate::{activity::{Activity, ActivityType, PreActivity}, competitor::Competitor, event::{ActivityIdentifier, Event}, matrix::CollisionMatrix, settings::Settings};

pub struct Master {
    pub competitors: Vec<Option<Competitor>>,
    pub activities: Vec<Activity>,
    activity_map: HashMap<(ActivityIdentifier, usize, ActivityType), usize>,
    pub collision_matrix: CollisionMatrix,
    fastest: HashMap<Event, usize>,
    settings: Settings,
    wcif: WcifContainer
}

impl Master {
    pub fn new(wcif: WcifContainer, gs: &str) -> Master {
        let mut competitors = wcif.persons_iter().filter(|p| p.registrant_id.is_some()).map(|p| Competitor::new(p, wcif.date())).collect::<Vec<_>>();
        let settings = Settings::new(&gs, competitors.len());
        let pre_activities = PreActivity::pre_activities(&wcif, &mut competitors, &settings);
        let mut activity_times_vec: Vec<_> = pre_activities.iter().map(|p| {
                (p.start, p.end, p.events.clone())
            })
            .collect();
        activity_times_vec.sort_unstable();
        let activities: Vec<_> = pre_activities.into_iter().flat_map(|p|p.into_activities(&settings)).collect();
        let collision_matrix = CollisionMatrix::new(activities.len());
        let activity_map = activities.iter().enumerate().flat_map(|(idx, act)| {
            act.id.iter().zip(std::iter::repeat((act.t.clone(), idx))).map(|(id, (t, idx))| ((id.clone(), act.group, t), idx)).collect::<Vec<_>>()
        }).collect();
        let mut fastest = HashMap::new();
        for (event, result) in competitors.iter().filter_map(|c|c.as_ref()).flat_map(|c|&c.pbs) {
            match fastest.get_mut(event) {
                None => {
                    fastest.insert(event.clone(), *result);
                }
                Some(v) => {
                    *v = (*v).min(*result);
                }
            }
        }
        let mut master = Master { 
            competitors,
            activities, 
            activity_map, 
            collision_matrix, 
            fastest, 
            settings,
            wcif
        };


        //Fix candidates in activities
        for act in master.activities.iter_mut() {
            match act.t {
                ActivityType::Competiter => {
                    act.candidates = act.preferred_candidates.clone();
                }
                ActivityType::Judge => {
                    act.candidates = master.competitors.iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_some() && c.as_ref().unwrap().available(&act.start, &act.end))
                        .filter(|(_, c)| !c.as_ref().unwrap().roles.contains(&Role::Delegate))
                        .filter(|(_, c)| act.id.iter().all(|e| c.as_ref().unwrap().qualified_judge(&e.event)))
                        .map(|(idx, _)| idx)
                        .collect();
                    act.candidates.grow(master.settings.id_no());
                    act.preferred_candidates.bitand_assign(act.candidates.clone());
                    assert!(!act.preferred_candidates.contains(0))
                }
                ActivityType::Scrambler => {
                    act.candidates = master.competitors.iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_some() && c.as_ref().unwrap().available(&act.start, &act.end))
                        .filter(|(_, c)| !c.as_ref().unwrap().roles.contains(&Role::Delegate))
                        .filter(|(_, c)| act.id.iter().all(|e| c.as_ref().unwrap().qualified_scrambler(&e.event)))
                        .map(|(idx, _)| idx)
                        .collect();
                    act.candidates.grow(master.settings.id_no());
                    act.preferred_candidates.bitand_assign(act.candidates.clone());
                }
            }
        }

        //Fix collision_matrix
        for i in 0..master.activities.len() - 1 {
            for j in i + 1..master.activities.len() {
                if master.activities[i].collides(&master.activities[j], &master.settings) {
                    master.collision_matrix.add_collision(i, j);
                }
            }
        }

        //Actual assignent woohoo
        //Competitors
        let mut current_end = activity_times_vec[0].1;
        let mut current_acts = vec![&activity_times_vec[0].2];
        for i in 1..activity_times_vec.len() {
            if activity_times_vec[i].0 < current_end {
                current_end = current_end.max(activity_times_vec[i].1);
                current_acts.push(&activity_times_vec[i].2);
            }
            else {
                master.assign(&current_acts, true);
                master.assign(&current_acts, false);

                current_end = activity_times_vec[i].1;
                current_acts = vec![&activity_times_vec[i].2];
            }
        }
        master.assign(&current_acts, true);
        master.assign(&current_acts, false);

        master.assign_t();

        for act in master.activities.iter() {
            assert_eq!(0, act.capacity);
        }

        /*for i in master.competitors.iter().filter_map(|x|x.as_ref()) {
            println!("{}: {}", i.name, i.debt);
        }*/

        /*for i in master.activities.iter() {
            if i.t == ActivityType::Competiter {
                println!("{:?}: {}", i.id, i.delegates_assinged);
                //println!("{:?}", i.assigned.ones().map(|x| &master.competitors[x].as_ref().unwrap().name).collect::<Vec<_>>());

            }
        }*/

        /*for i in master.competitors[6].as_ref().unwrap().assignemtns.ones() {
            let act = &master.activities[i];
            println!("{:?} {:?} {:?} {:?}", act.start, act.end, act.id, act.t);
        }*/

        master
    }

    fn assign_t(&mut self) {
        loop {
            let critical = self.activities.iter().enumerate()
                .filter(|(_, act)| act.capacity > 0)
                .min_by(|(_, act1), (_, act2)| act1.available_leftover().partial_cmp(&act2.available_leftover()).unwrap());
            let (idx, id) = match critical {
                None => break,
                Some((idx, act)) => {
                    println!("{:?} {:?}", act.id, act.t);
                    (idx, if act.preferred_candidates.count_ones(..) > 0 {
                        act.preferred_candidates.ones()
                            .map(|act| (act, self.competitors[act].as_ref().unwrap().debt))
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
                    }
                    else {
                        act.candidates.ones()
                            .map(|act| (act, self.competitors[act].as_ref().unwrap().debt))
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
                    })
                }
            };
            self.assign_comp(&vec![idx], id);
            self.competitors[id].as_mut().unwrap().assign(idx);
            let cost: f32 = self.activities[idx].id.iter().map(|e| self.settings.staff_multiplier(&e.event)).sum();
            self.competitors[id].as_mut().unwrap().debt -= cost;
        }
    }

    pub fn is_fast(&self, event: &Event, id: usize) -> bool {
        event.id() != "333mbf" &&
        event.id() != "666" &&
        event.id() != "777" &&
        event.id() != "minx" && 
        self.competitors[id].as_ref().map(|s|(*s.pb(event).unwrap_or_else(||&usize::MAX) as f32) < self.fastest[event] as f32 * 1.25).unwrap_or_else(||false)
    }

    fn acts(&self, act: ActivityIdentifier, t: ActivityType) -> Vec<usize> {
        let mut ret = vec![];
        for i in 0.. {
            match self.activity_map.get(&(act.clone(), i, t.clone())) {
                None => break,
                Some(v) => ret.push(*v)
            }
        }
        ret
    }

    fn assign(&mut self, acts: &Vec<&Vec<ActivityIdentifier>>, delegate: bool) {
        let mut allready_assigned = FixedBitSet::new();
        let mut acts = acts.iter()
            .map(|x| {
                (*x, None)
            })
            .collect();
        self.assign_internal(vec![], vec![], 0, &mut acts, &mut allready_assigned, delegate);
    }

    fn assign_internal(&mut self, combs: Vec<Vec<usize>>, idxs: Vec<usize>, idx: usize, acts: &mut Vec<(&Vec<ActivityIdentifier>, Option<usize>)>, allready_assigned: &mut FixedBitSet, delegate: bool) {
        if idx == acts.len() {
            //Checks wether this is the combination of no events, in which case we do nothing. The code would panic if we did not check.
            if idxs.len() != 0 {
                let fast = combs.last().unwrap();
                if fast.len() == acts.len() {
                    for (idx, (act, opt)) in acts.iter_mut().enumerate() {
                        if self.is_final(&act[0].event) && opt.is_none() {
                            *opt = Some(fast[idx]);
                        }
                    }
                }

                let to_be_assigned = fast.iter().map(|x| {
                    let act = &self.activities[*x];
                    act.candidates.clone()
                }).reduce(|a, b| a.bitand(&b)).unwrap();
                for id in to_be_assigned.difference(allready_assigned)
                    .filter(|id| {
                        delegate == self.competitors[*id].as_ref().unwrap().roles.contains(&Role::Delegate)
                    }).collect::<Vec<_>>().into_iter()
                    {
                    let new_combs = combs.iter()
                        .filter(|comb| {
                            idxs.iter().enumerate().all(|(i, idx)| {
                                let (act, fast) = acts[*idx];
                                match fast {
                                    Some(v) => !act.iter().all(|act| self.is_fast(&act.event, id)) || comb[i] == v,
                                    _ => true
                                }
                            })
                        });
                    let comb = if self.competitors[id].as_ref().unwrap().roles.contains(&Role::Delegate) {
                        self.max_comb_delegate(new_combs)
                    } 
                    else {
                        self.max_comb(new_combs)
                    };
                    let comb = comb.clone();
                    self.assign_comp(&comb, id);
                }

                allready_assigned.bitor_assign(&to_be_assigned);
            }
        }
        else {
            let mut new_idxs = idxs.clone();
            new_idxs.push(idx);
            let ids = self.acts(acts[idx].0[0].clone(), ActivityType::Competiter);
            let new_combs: Vec<_> = if combs.len() != 0 { 
                combs.iter().flat_map(|comb| {
                    ids.iter().zip(std::iter::repeat(comb.to_owned()))
                        .filter(|(new_id, comb)| {
                            comb.iter().all(|id| !self.collision_matrix.does_collide(*id, **new_id))
                        })
                        .map(|(new_id, mut comb)| {
                            comb.push(*new_id);
                            comb
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
            } else {
                ids.into_iter().map(|x|vec![x]).collect()
            };
            self.assign_internal(new_combs, new_idxs, idx + 1, acts, allready_assigned, delegate);
            self.assign_internal(combs, idxs, idx + 1, acts, allready_assigned, delegate);
        }
    }

    fn is_final(&self, event: &Event) -> bool {
        self.wcif.events_iter().find(|e|&e.id == event.id()).unwrap().rounds.len() == 1
    }

    fn max_comb<'a>(&self, combs: impl Iterator<Item = &'a Vec<usize>> + 'a) -> &'a Vec<usize> {
        combs.max_by_key(|x| self.comb_cap(x)).unwrap()
    }

    fn max_comb_delegate<'a>(&self, combs: impl Iterator<Item = &'a Vec<usize>> + 'a) -> &'a Vec<usize> {
        combs.filter(|a| self.comb_cap(a) > 0).min_by_key(|a| self.delegate_comp(a)).unwrap()
    }

    fn delegate_comp(&self, comb: &Vec<usize>) -> (usize, usize) {
        let max = comb.iter().map(|x| self.activities[*x].delegates_assinged).max().unwrap();
        let sum = comb.iter().map(|x| self.activities[*x].delegates_assinged).sum();
        (max, sum)
    }

    fn comb_cap(&self, comb: &Vec<usize>) -> usize {
        comb.iter().map(|x| self.activities[*x].capacity).min().unwrap()
    }

    fn assign_comp(&mut self, comb: &Vec<usize>, id: usize) {
        for act in comb {
            if self.competitors[id].as_ref().unwrap().roles.contains(&Role::Delegate) {
                self.activities[*act].assign_delegate(id);
            }
            else {
                self.activities[*act].assign(id);
            }
            self.competitors[id].as_mut().unwrap().assign(*act);
            for col_act in self.collision_matrix.collision_set(*act).ones() {
                self.activities[col_act].remove_candidate(id);
            }
        }
    }
}