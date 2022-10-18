use super::*;

use super::DB;

use scorecard_to_pdf::Return;
use wca_oauth::{Assignment, AssignmentCode};

pub fn is_localhost(socket: Option<SocketAddr>) -> Result<(), Rejection> {
    if let Some(socket) = socket {
        let ip = socket.ip();
        match ip {
            std::net::IpAddr::V4(ip) if ip == std::net::Ipv4Addr::LOCALHOST => return Ok(()),
            _ => ()
        }
    }
    Err(warp::reject())
}

pub async fn root(db: DB, id: String, query: HashMap<String, String>, socket: Option<SocketAddr>) -> Result<Response<String>, Rejection> {
    is_localhost(socket)?;
    let auth_code = &query["code"];
    let oauth = wca_oauth::OAuth::get_auth(
        "TDg_ARkGANTJB_z0oeUWBVl66a1AYdYAxc-jPJIhSfY".to_owned(), 
        "h0jIi8YkSzJo6U0JRQk-vli21yJ58kuz7_p9-AUyat0".to_owned(), 
        "http://localhost:5000/".to_owned(), 
        auth_code.to_owned()).await;
    let json = oauth.get_wcif(&id).await;
    let body = match json {
        Ok(mut json) => {
            let body = event_list_to_html(get_rounds(&mut json)).to_string();
            let mut db_guard = db.lock().await;
            *db_guard = Some(json.add_oauth(oauth));
            drop(db_guard);
            body
        }
        Err(err) => format!("Failed to load data for competition. Encontured following error: {}", err.error)
    };
    
    Response::builder()
        .header("content-type", "text/html")
        .body(body)
        .map_err(|_| warp::reject())
}

pub async fn round(db: DB, query: HashMap<String, String>, socket: Option<SocketAddr>) -> Result<Response<String>, Rejection> {
    is_localhost(socket)?;
    let eventid = &query["eventid"];
    let round = usize::from_str_radix(&query["round"], 10).unwrap();
    let mut db_guard = db.lock().await;
    let wcif = (*db_guard).as_mut().unwrap();
    let (competitors, map) = crate::wcif::get_competitors_for_round(wcif, eventid, round);
    drop(db_guard);
    let str = competitors.iter()
        .rev()
        .map(|id|{
            format!("{}\\r{}", id, map[id])
        })
        .collect::<Vec<_>>()
        .join("\\n");
    Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(crate::compiled::js_replace(&str, competitors.len(), eventid, round))
        .map_err(|_| warp::reject())
}

pub async fn pdf(db: DB, query: HashMap<String, String>, socket: Option<SocketAddr>, stages: Option<Stages>) -> Result<Response<Vec<u8>>, Rejection> {
    is_localhost(socket)?;
    let eventid = &query["eventid"];
    let round = query["round"].parse().unwrap();
    let group = &query["groups"];
    let wcif = query["wcif"].parse().unwrap();
    let wcif_oauth = &mut db.lock().await;
    let groups: Vec<Vec<_>> = group.split("$")
        .map(|group|{
            group.split("s")
                .map(str::parse)
                .filter_map(Result::ok)
                .collect()
        })
        .collect();

    let wcif_oauth = wcif_oauth.as_mut().unwrap();
    if wcif {
        match wcif_oauth.add_groups_to_event(eventid, round, groups.len()) {
            Ok(activities) => {
                let activity_ids: Vec<_> = activities.into_iter().map(|act| act.id).collect();
                for (group, activity_id) in groups.iter().zip(activity_ids) {
                    for (station, id) in group.into_iter().enumerate() {
                        //This runs in O(nm) time which is horrible, when it could run in O(n) time but n and m are both small so i will let it be for now :)
                        wcif_oauth.patch_persons(|person|{
                            if person.registrant_id == Some(*id) {
                                //Stations are not correctly evaluated if multiple stages are used so station assignment needs to be moved to here instead of in run_from_wcif.
                                person.assignments.push(Assignment { activity_id, assignment_code: AssignmentCode::Competitor, station_number: Some(station + 1) })
                            }
                        })
                    }
                }
                wcif_oauth.patch().await;
            }
            Err(()) => println!("Unable to patch likely because the given event already has groups in the wcif."),
        }
    }

    let bytes = crate::pdf::run_from_wcif(wcif_oauth, eventid, round, groups, &stages);

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