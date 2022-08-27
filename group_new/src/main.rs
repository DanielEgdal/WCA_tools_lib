pub mod event;
mod competitor;
pub mod matrix;
mod activity;
mod settings;
mod master;

fn main() {
    let wcif = wca_oauth::parse(std::fs::read_to_string("wcif.json").unwrap()).unwrap();
    let gs = std::fs::read_to_string("settings.gs").unwrap();
    let now = std::time::Instant::now();
    let master = master::Master::new(wcif, &gs);
    let time = now.elapsed();
    println!("{:?}", time);

    let (mut activities, competitors) = (master.activities, master.competitors);
    activities.sort_by(|a, b| a.id.cmp(&b.id).then(a.group.cmp(&b.group).then(a.t.cmp(&b.t))));
    for act in activities {
        println!("{}, {:?}, {:?}, {:?}", act.id.iter().map(|x|x.event.id()).collect::<Vec<_>>().join("/"), act.group, act.t, act.assigned.ones().map(|x| &competitors[x].as_ref().unwrap().name).collect::<Vec<_>>())
    }
    /*for act in master.activities {
        println!("{:?}", act);
    }
    println!("{:?}", master.collision_matrix);*/
}
