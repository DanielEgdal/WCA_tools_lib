use super::*;

use super::DB;

use scorecard_to_pdf::Return;

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
    let wcif_oauth = &mut db.lock().await;
    let groups: Vec<Vec<_>> = group.split("$")
        .map(|group|{
            group.split("s")
                .map(str::parse)
                .filter_map(Result::ok)
                .collect()
        })
        .collect();
    let bytes = crate::pdf::run_from_wcif(wcif_oauth.as_mut().unwrap(), eventid, round, groups, &stages).await;

    //wcif_oauth.as_mut().unwrap().activity_iter().for_each(|act|{
    //    println!("{:?}", act);
    //});

    //let str = wcif_oauth.as_mut().unwrap().patch().await;

    //println!("{}", str);

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