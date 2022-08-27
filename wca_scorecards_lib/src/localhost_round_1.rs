use std::{collections::HashMap, net::SocketAddr};
use crate::pdf::run;
use scorecard_to_pdf::Language;
use warp::{Filter, hyper::Response, Rejection};
use scorecard_to_pdf::Return;

#[tokio::main]
pub async fn init(id: String, groups_csv: String, limit_csv: String) {
    //Url to approve the Oauth application
    let auth_url = "https://www.worldcubeassociation.org/oauth/authorize?client_id=TDg_ARkGANTJB_z0oeUWBVl66a1AYdYAxc-jPJIhSfY&redirect_uri=http%3A%2F%2Flocalhost%3A5000%2F&response_type=code&scope=public+manage_competitions";
    
    //Handling the get request from authentification. HTTP no s, super secure, everything is awesome. The API said that https is not required for localhost so it is fine.
    let root = warp::path::end()
        .and(warp::query::query())
        .and(warp::addr::remote())
        .and_then(move |query: HashMap<String, String>, socket: Option<SocketAddr>| {
            let id = id.clone();
            let groups_csv = groups_csv.clone();
            let limit_csv = limit_csv.clone();
            root(groups_csv, limit_csv, id, query, socket)
        });

    //Try opening in browser. In case of fail write the url to the terminal
    match open::that(auth_url) {
        Err(_) => {
            println!("Please open the following website and follow the instructions:");
            println!("{}", auth_url);
        }
        Ok(_) => ()
    }

    warp::serve(root).run(([127, 0, 0, 1], 5000)).await;
}

async fn root(groups_csv: String, limit_csv: String, id: String, query: HashMap<String, String>, socket: Option<SocketAddr>) -> Result<Response<Vec<u8>>, Rejection> {
    crate::localhost::is_localhost(socket)?;
    let auth_code = &query["code"];
    let oauth = wca_oauth::OAuth::get_auth(
        "TDg_ARkGANTJB_z0oeUWBVl66a1AYdYAxc-jPJIhSfY".to_owned(), 
        "h0jIi8YkSzJo6U0JRQk-vli21yJ58kuz7_p9-AUyat0".to_owned(), 
        "http://localhost:5000/".to_owned(), 
        auth_code.to_owned()).await;
    let json = oauth.get_wcif(&id).await;
    let body = match json {
        Ok(json) => {
            let mut wcif = json.add_oauth(oauth);
            let name = wcif.get().name.clone();
            let pdf = run(&groups_csv, &limit_csv, &name, Language::english(), Some(&mut wcif), None);
            wcif.patch().await;
            match pdf {
                Return::Pdf(b) => b,
                Return::Zip(b) => b
            }
        }
        Err(err) => format!("Failed to load data for competition. Encontured following error: {}", err.error).as_bytes().to_vec()
    };
    
    Response::builder()
        .header("content-type", "application/pdf")
        .body(body)
        .map_err(|_| warp::reject())
}