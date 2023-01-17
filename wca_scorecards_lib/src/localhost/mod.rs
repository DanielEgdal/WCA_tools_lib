use std::{sync::Arc, collections::HashMap, net::SocketAddr};
use crate::{wcif::*, Stages};
use tokio::sync::Mutex;
use warp::{Filter, hyper::Response, Rejection};
use wca_oauth::WcifOAuth;

mod html;
mod responses;

use html::event_list_to_html;
use responses::*;

pub use responses::is_localhost;

type DB = Arc<Mutex<Option<WcifOAuth>>>;

#[tokio::main]
pub async fn init(id: String, stages: Stages) {
    //Url to approve the Oauth application
    let auth_url = "https://www.worldcubeassociation.org/oauth/authorize?client_id=nqbnCQGGO605D_XYpgghZdIN2jDT67LhhUC1kE-Msuk&redirect_uri=http%3A%2F%2Flocalhost%3A5000%2F&response_type=token&scope=public+manage_competitions";

    //Mutex for storing the authentification code for async reasons.
    let wcif: DB = Arc::new(Mutex::new(None));
    
    //Handling the get request from authentification. HTTP no s, super secure, everything is awesome. The API said that https is not required for localhost so it is fine.
    let local_wcif = wcif.clone();
    let root = warp::path::end()
        .and(warp::query::query())
        .and(warp::addr::remote())
        .and_then(move |query: HashMap<String, String>, socket: Option<SocketAddr>| {
            let id = id.clone();
            let wcif = local_wcif.clone();
            root(wcif, id, query, socket)
        });

    //Get request for specific round. Query to specify which event and round is to be used.
    let local_wcif = wcif.clone();
    let group_size = stages.capacity * stages.no;
    let round = warp::path!("round")
        .and(warp::query::query())
        .and(warp::addr::remote())
        .and_then(move |query: HashMap<String,String>, socket: Option<SocketAddr>|{
            let wcif = local_wcif.clone();
            round(wcif, query, socket, group_size)
        });

    //Get request for pdf. Query to specify which event, round and groups to be used.
    let local_wcif = wcif.clone();
    let pdf = warp::path!("round" / "pdf")
        .and(warp::query::query())
        .and(warp::addr::remote())
        .and_then(move |query: HashMap<String, String>, socket: Option<SocketAddr>|{
            let wcif = local_wcif.clone();
            let stages = stages.clone();
            pdf(wcif, query, socket, stages)
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

    let routes = root
        .or(round)
        .or(pdf)
        .or(wasm_js)
        .or(js)
        .or(wasm);

    //Try opening in browser. In case of fail write the url to the terminal
    match open::that(auth_url) {
        Err(_) => {
            println!("Please open the following website and follow the instructions:");
            println!("{}", auth_url);
        }
        Ok(_) => ()
    }

    warp::serve(routes).run(([127, 0, 0, 1], 5000)).await;
}