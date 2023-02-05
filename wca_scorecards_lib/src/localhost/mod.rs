use std::{collections::HashMap};
use crate::{wcif::*, Stages, ScorecardOrdering};
use warp::{Filter, hyper::Response, Rejection};

mod html;
mod responses;
mod db;

use html::event_list_to_html;
use responses::*;

pub use db::DB;

#[tokio::main]
pub(crate) async fn init(stages: Stages, compare: ScorecardOrdering) {
    //Mutex for storing the authentification code for async reasons.
    let wcif: DB = DB::new();

    let client_id = "nqbnCQGGO605D_XYpgghZdIN2jDT67LhhUC1kE-Msuk";

    let redirect_uri = "http://localhost:5000/validated";
    
    let validate = warp::path::end()
        .and_then(validate);

    //Handling the get request from authentification. HTTP no s, super secure, everything is awesome. The API said that https is not required for localhost so it is fine.
    let root = warp::path!("validated")
        .and(warp::query::query())
        .and_then(move |query: HashMap<String, String>| {
            root(query, redirect_uri.to_string(), client_id.to_string())
        });

    //Competition request
    let local_wcif = wcif.clone();
    let competition = warp::path!("competition")
        .and(warp::query::query())
        .and_then(move |query: HashMap<String, String>| {
            let db = local_wcif.clone();
            competition(db, query, client_id.to_string(), redirect_uri.to_string())
        });

    //Get request for specific round. Query to specify which event and round is to be used.
    let local_wcif = wcif.clone();
    let group_size = stages.capacity * stages.no;
    let round = warp::path!("round")
        .and(warp::query::query())
        .and_then(move |query: HashMap<String,String>,|{
            let wcif = local_wcif.clone();
            round(wcif, query, group_size)
        });

    //Get request for pdf. Query to specify which event, round and groups to be used.
    let local_wcif = wcif.clone();
    let pdf = warp::path!("round" / "pdf")
        .and(warp::query::query())
        .and_then(move |query: HashMap<String, String>|{
            let wcif = local_wcif.clone();
            let stages = stages.clone();
            let client_id = client_id.to_string();
            let redirect_uri = redirect_uri.to_string();
            pdf(wcif, query, stages, compare, client_id, redirect_uri)
        });

    let wasm_js = warp::path!("round" / "pkg" / "group_menu.js")
        .map(|| Response::builder()
        .header("content-type", "text/javascript")
        .body(crate::compiled::WASM_JS));

    let js = warp::path!("round" / "pkg" / "snippets" / "group_menu-c33353fa00f3dafb" / "src" / "js.js")
        .map(|| Response::builder()
        .header("content-type", "text/javascript")
        .body(crate::compiled::JS));
    
    let wasm = warp::path!("round" / "pkg" / "group_menu_bg.wasm")
        .map(|| Response::builder()
        .header("content-type", "text/wasm")
        .body(crate::compiled::WASM));

    let routes = validate
        .or(root)
        .or(competition)
        .or(round)
        .or(pdf)
        .or(wasm_js)
        .or(js)
        .or(wasm);

    let serve = warp::serve(routes).run(([127, 0, 0, 1], 5000));

    let mut interval = async_timer::Interval::platform_new(core::time::Duration::from_secs(600));

    let garbage_eliminator = async {
        let mut wcif = wcif.clone();
        loop {
            wcif.garbage_elimination().await;
            interval.as_mut().await;
        }
    };

    std::future::join!(serve, garbage_eliminator).await;
}

