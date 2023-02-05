use super::*;

use super::DB;

use scorecard_to_pdf::Return;
use wca_oauth::{Assignment, AssignmentCode};

pub fn is_localhost(socket: Option<SocketAddr>) -> Result<(), Rejection> {
    Ok(())
    /*if let Some(socket) = socket {
        let ip = socket.ip();
        match ip {
            std::net::IpAddr::V4(ip) if ip == std::net::Ipv4Addr::LOCALHOST => return Ok(()),
            _ => ()
        }
    }
    Err(warp::reject())*/
}

pub async fn root(db: DB, id: String, query: HashMap<String, String>, redirect_uri: String, client_id: String, socket: Option<SocketAddr>) -> Result<Response<String>, Rejection> {
    is_localhost(socket)?;
    if !query.contains_key("access_token") {
        return Response::builder()
            .header("content-type", "text/html")
            // Why did anyone think it was a good idea to use a data fragment instead of a query when it is a query.
            // This has caused so much pain and i hate this "solution"
            // The world is covered in idiots who do not think one bit when designing their software (including me)
            // I could keep on ranting forever but whatever i will stop now.
            // Anyway this works by replacing the location, i.e. the stuff after the path in the url
            // with the location hash i.e. the data fragment but first replacing the leading hash with a question mark.
            // This means that we actually get this request twice, but only javascript has access to this data so this we must.
            // And to make matters worse javascript is so slow at initializing, so this just makes it painfully slow.
            // I am considering going back to exposing my secret just so i do not have to do this. I hate this so much.
            .body("<script>window.location.replace(window.location.hash.replace(\"#\",\"?\"))</script>".to_string())
            .map_err(|_| warp::reject())
    }
    let auth_token = &query["access_token"];
    let oauth = wca_oauth::OAuth::get_auth_implicit(
        client_id.into(), 
        auth_token.into(), 
        redirect_uri).await;

    Response::builder()
        .header("content-type", "text/html")
        .body(format!("<a href=\"competition?auth_code={}&competition=dsfgeneralforsamlingen2023\">link</a>", auth_token))
        .map_err(|_| warp::reject())
}

pub async fn competition(mut db: DB, query: HashMap<String, String>, client_id: String, redirect_uri: String) -> Result<Response<String>, Rejection> {
    let auth_code = &query["auth_code"];
    let competition = &query["competition"];
    let oauth = wca_oauth::OAuth::get_auth_implicit(client_id, auth_code.to_string(), redirect_uri).await;
    let json = oauth.get_wcif(&competition).await;
    let body = match json {
        Ok(mut json) => {
            let body = event_list_to_html(get_rounds(&mut json), &auth_code, competition).to_string();
            db.insert_wcif(competition.to_string(), auth_code.to_string(), json).await;
            body
        }
        Err(err) => format!("Failed to load data for competition. Encontured following error: {}", err.error)
    };
    
    Response::builder()
        .header("content-type", "text/html")
        .body(body)
        .map_err(|_| warp::reject())
}


pub async fn round(mut db: DB, query: HashMap<String, String>, socket: Option<SocketAddr>, group_size: u32) -> Result<Response<String>, Rejection> {
    is_localhost(socket)?;
    let eventid = &query["eventid"];
    let auth_code = &query["auth_code"];
    let competition = &query["competition"];
    let round = query["round"].parse().unwrap();
    let mut wcif_lock = db.get_wcif_lock(competition.clone(), auth_code.clone()).await;
    let wcif = wcif_lock.get();

    let response = match wcif {
        Some(wcif) => {     
            let (competitors, map) = crate::wcif::get_competitors_for_round(wcif, eventid, round); 
            let str = competitors.iter()
                .rev()
                .map(|id|{
                    format!("{}\\r{}", id, map[id])
                })
                .collect::<Vec<_>>()
                .join("\\n");
            crate::compiled::js_replace(&str, competitors.len(), eventid, round, group_size, auth_code, competition)
        },
        None => format!("The competition has not been loaded."),
    };
    
    Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(response)
        .map_err(|_| warp::reject())
}

pub(crate) async fn pdf(mut db: DB, query: HashMap<String, String>, socket: Option<SocketAddr>, stages: Stages, compare: ScorecardOrdering, client_id: String, redirect_uri: String) -> Result<Response<Vec<u8>>, Rejection> {
    fn assign_stages(groups: Vec<Vec<usize>>, stages: &Stages) -> Vec<Vec<(usize, usize)>> {
        groups.into_iter()
            .map(|group| {
                    let no_of_stages = (group.len() + stages.capacity as usize - 1) / stages.capacity as usize;
                    let lower_per_stage = group.len() / no_of_stages;
                    let leftover = group.len() - lower_per_stage * no_of_stages;
                    let splits = (0..no_of_stages).map(|i| lower_per_stage * i + i.min(leftover));
                    group.into_iter().enumerate().map(|(idx, id)| {
                        let (stage, lower) = splits.clone().enumerate().rev().find(|(_, lower)| *lower <= idx).expect("First is 0");
                        let station = stages.capacity as usize * stage + idx - lower + 1;
                        (id, station)
                    }).collect()  
                })
            .collect()
    }
    
    is_localhost(socket)?;
    let eventid = &query["eventid"];
    let round = query["round"].parse().unwrap();
    let group = &query["groups"];
    let wcif = query["wcif"].parse().unwrap();
    let competition = &query["competition"];
    let auth_code = &query["auth_code"];
    let oauth = wca_oauth::OAuth::get_auth_implicit(client_id, auth_code.clone(), redirect_uri).await;

    let groups: Vec<Vec<_>> = group.split("$")
        .map(|group|{
            group.split("s")
                .map(str::parse)
                .filter_map(Result::ok)
                .collect()
        })
        .collect();

    let groups_with_stations = assign_stages(groups.clone(), &stages);
    let mut db_lock = db.get_wcif_lock(competition.clone(), auth_code.clone()).await;
    let mut wcif_oauth = db_lock.get().unwrap();
    if wcif {
        match wcif_oauth.add_groups_to_event(eventid, round, groups.len()) {
            Ok(activities) => {
                let activity_ids: Vec<_> = activities.into_iter().map(|act| act.id).collect();
                for (group, activity_id) in groups_with_stations.iter().zip(activity_ids) {
                    for (id, station) in group.into_iter() {
                        //This runs in O(nm) time which is horrible, when it could run in O(n) time but n and m are both small so i will let it be for now :)
                        wcif_oauth.patch_persons(|person|{
                            if person.registrant_id == Some(*id) {
                                person.assignments.push(Assignment { activity_id, assignment_code: AssignmentCode::Competitor, station_number: Some(*station) })
                            }
                        })
                    }
                }
                let response = wcif_oauth.patch(&oauth).await;
                println!("Patched to wcif. Received the following response: \n{}", response);
            }
            Err(()) => println!("Unable to patch likely because the given event already has groups in the wcif."),
        }
    }

    let bytes = crate::pdf::run_from_wcif(&mut wcif_oauth, eventid, round, groups_with_stations, &stages, compare);

    match bytes {
        Return::Pdf(bytes) => {
            Response::builder()
                .header("content-type", "application/pdf")
                .body(bytes)
                .map_err(|_| warp::reject())
        }
        Return::Zip(bytes) => {
            Response::builder()
                .header("content-type", "application/zip")                
                .body(bytes)
                .map_err(|_| warp::reject())
        }
    }
}
