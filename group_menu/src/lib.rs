use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js.js")]
extern {
    pub fn log(string: &str);
    pub fn add_group(group: usize);
    pub fn add_name(str: &str, group: usize, index: usize, groups: usize);
    pub fn onclick(id: &str, f: &Closure<dyn Fn()>);
    pub fn move_td(o_group: usize, o_index: usize, d_group: usize, d_index: usize, groups: usize);
    pub fn group_len(group: usize, groups: usize) -> usize;
    pub fn update_id(id: &str, new_id: &str);
    pub fn href(id: &str, str: &str);
}

#[wasm_bindgen]
extern {
    pub fn prompt(str: &str) -> String;
}

type Groups = Vec<Vec<(usize, String, usize)>>;

static mut GROUPS: Groups = vec![];

#[wasm_bindgen]
pub fn initialize(data: &str, number: usize) {
    let tables = prompt("How many solving stations will you be using?");
    let max_group_size = match usize::from_str_radix(&tables, 10) {
        Err(_) => try_again(),
        Ok(v) => v
    };
    let groups = (number + max_group_size - 1) / max_group_size;
    let competitors = data
        .split("\n")
        .enumerate()
        .map(|(index, str)| {
            let mut iter = str.split("\r");
            let id = usize::from_str_radix(iter.next().unwrap(), 10).unwrap();
            let name = iter.next().unwrap();
            (number - index, name, id)
        });
    let groups_data: Groups = GroupIter::new(competitors, number, max_group_size)
        .enumerate()    
        .map(|(group, vec)|{
            add_group(group + 1);
            vec.into_iter()
                .rev()
                .enumerate()
                .map(|(index, (placement, name, id))|{
                    add_name(&format!("{}: {}({})", placement, name, id), group, index, groups);
                    if group > 0 { 
                        let closure = Closure::new(move || move_td_all(group, index, -1, groups));
                        onclick(&format!("g{}i{}l", group, index), &closure);
                        closure.forget();
                    }
                    if group < groups - 1 {
                        let closure = Closure::new(move || move_td_all(group, index, 1, groups));
                        onclick(&format!("g{}i{}r", group, index), &closure);
                        closure.forget();
                    }
                    (placement, name, id)
                })
                .collect()
        })
        .collect();
    unsafe { GROUPS = groups_data; }
    href("submit", &groups_to_string());
}

fn move_td_all(group: usize, index: usize, direction: isize, groups: usize) {
    let this_group_len = unsafe { GROUPS[group].len() };
    move_td_all_sub(group, index, direction, groups);
    for i in index + 1.. {
        if i >= this_group_len {
            break;
        }
        move_td_all_sub(group, i, 0, groups);
    }
    unsafe {
        let this = GROUPS[group].remove(index);
        GROUPS[group + direction as usize].push(this);    
    }
    href("submit", &groups_to_string());
}

fn groups_to_string() -> String {
    unsafe { 
        GROUPS.iter()
            .map(|group|{
                group.iter()
                    .map(|(_, _, id)|{
                        id.to_string()
                    })
                    .collect::<Vec<_>>()
                    .join("s")
            })
            .collect::<Vec<_>>()
            .join("$") 
    }
}

fn move_td_all_sub(group: usize, index: usize, direction: isize, groups: usize) {
    let group = 3 * group;
    let new_group = group + 3 * direction as usize;
    let new_group_len = group_len(new_group + 1, 3 * groups);
    for i in 0..3 {
        move_td(group + i, index, new_group + i, new_group_len, 3 * groups);
    }
    let c = |d| {
        let closure = Closure::new(move || move_td_all(new_group / 3, new_group_len, d, groups));
        let s = if d == 1 {'r'} else {'l'};
        let id = format!("g{}i{}{}", group / 3, index, s);
        let new_id = format!("g{}i{}{}", new_group / 3, new_group_len, s);
        update_id(&id, &new_id);
        onclick(&new_id, &closure);
        closure.forget();
    };
    if new_group != 0 { c(-1); }
    if new_group != 3 * (groups - 1) { c(1); }
}

struct GroupIter<'a, I> where I: Iterator<Item = (usize, &'a str, usize)> {
    iter: I,
    group_size: usize,
    modulo: usize, 
    acc: usize,
    curr_group: usize
}

impl<'a, I> GroupIter<'a, I> where I: Iterator<Item = (usize, &'a str, usize)> {
    fn new(iter: I, number: usize, max_group_size: usize) -> Self {
        let number_of_groups = (number + max_group_size - 1) / max_group_size;
        let group_size = number / number_of_groups;
        let modulo = number % number_of_groups;
        let acc = 0;
        Self { iter, group_size, modulo, acc, curr_group: 0 }
    }
}

impl<'a, I> Iterator for GroupIter<'a, I> where I: Iterator<Item = (usize, &'a str, usize)> {
    type Item = Vec<(usize, String, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut vec = vec![];
        loop {
            let next = self.iter.next();
            if let Some((index, name, id)) = next {
                vec.push((index, name.to_string(), id));
                let this_group_size = self.group_size + match self.modulo > self.curr_group {
                    true => 1,
                    false => 0
                };
                self.acc += 1;
                if self.acc == this_group_size {
                    self.acc = 0;
                    self.curr_group += 1;
                    break;
                }
            }
            else {
                if vec.len() == 0 {
                    return None;
                }
                else {
                    break;
                }
            }
        }
        Some(vec)
    }
}

pub fn try_again() -> usize {
    let tables = prompt("Invalid input. How many solving stations will you be using?");
    match usize::from_str_radix(&tables, 10) {
        Err(_) => try_again(),
        Ok(v) => v
    }
}