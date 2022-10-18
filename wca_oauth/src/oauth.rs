use core::panic;
use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;
use sha2::{Sha512, Digest};
use crate::*;
use std::io::{Write, Read};

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String
}

#[derive(Debug)]
pub struct OAuth {
    access_token: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    client: Client
}

impl OAuth {
    pub async fn get_auth_with_password(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let mut oauth = Self {
            access_token: format!(""),
            refresh_token: format!(""),
            client_id,
            client_secret,
            redirect_uri,
            client: reqwest::Client::new()
        };

        let refresh = std::fs::read("refresh");
        match refresh {
            Ok(data) => {
                println!("Please enter your password:");
                std::io::stdout().flush().unwrap();
                let password = rpassword::read_password().unwrap();
                let decrypted = xor_password(&password, &data);
                match String::from_utf8(decrypted) {
                    Ok(refresh_token) => {
                        oauth.refresh_token = refresh_token;
                        oauth.refresh_auth().await;
                        let encrypted = xor_password(&password, &oauth.refresh_token.clone().as_bytes().to_vec());
                        let mut file = std::fs::File::create("refresh").unwrap();
                        file.write_all(&encrypted).expect("I do not know what the problem is");
                    }
                    Err(_) => panic!("Incorrect password. Please restart")
                }
            }
            Err(_) => {
                println!("Please enter a valid auth code for the given client:");
                let mut code = String::new();
                std::io::stdin().read_line(&mut code).expect("OS error 97");
                let auth = oauth.get_auth_normal_flow(code.trim().to_string());
                println!("Please enter a password:");
                std::io::stdout().flush().unwrap();
                let password = rpassword::read_password().unwrap();
                let mut file = std::fs::File::create("refresh").unwrap();
                auth.await;
                let encrypted = xor_password(&password, &oauth.refresh_token.clone().as_bytes().to_vec());
                file.write_all(&encrypted).expect("I do not know what the problem is");
            }
        };
        oauth
    }

    pub async fn get_auth(client_id: String, client_secret: String, redirect_uri: String, auth_code: String) -> Self {
        let mut oauth = Self {
            access_token: format!(""),
            refresh_token: format!(""),
            client_id,
            client_secret,
            redirect_uri,
            client: reqwest::Client::new()
        };
        oauth.get_auth_normal_flow(auth_code).await;
        oauth
    }


    async fn get_auth_normal_flow(&mut self, code: String) {
        let mut params = HashMap::new();

        params.insert("grant_type", "authorization_code");
        params.insert("client_id", &self.client_id);
        params.insert("client_secret", &self.client_secret);
        params.insert("redirect_uri", &self.redirect_uri);
        params.insert("code", &code.trim());

        //Request token
        let response = self.client
            .post("https://www.worldcubeassociation.org/oauth/token")
            .form(&params)
            .send()
            .await.unwrap()
            .text()
            .await.unwrap();

        let auth_response: AuthResponse = serde_json::from_str(&response).unwrap();

        self.access_token = auth_response.access_token;
        self.refresh_token = auth_response.refresh_token;
    }

    async fn refresh_auth(&mut self) {
        let mut params = HashMap::new();

        params.insert("grant_type", "refresh_token");
        params.insert("client_id", "TDg_ARkGANTJB_z0oeUWBVl66a1AYdYAxc-jPJIhSfY");
        params.insert("client_secret", "h0jIi8YkSzJo6U0JRQk-vli21yJ58kuz7_p9-AUyat0");
        params.insert("refresh_token", &self.refresh_token.trim());

        //Request token
        let response = self.client
            .post("https://www.worldcubeassociation.org/oauth/token")
            .form(&params)
            .send()
            .await.unwrap()
            .text()
            .await.unwrap();

        let auth_response: AuthResponse = serde_json::from_str(&response).unwrap();

        self.access_token = auth_response.access_token;
        self.refresh_token = auth_response.refresh_token;
    }

    pub async fn get_wcif_api(self: &Self, id: &str) -> String {
        let get_url = format!("https://www.worldcubeassociation.org/api/v0/competitions/{}/wcif", id);
        //Request wcif
        let response = self.client
            .get(&get_url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await.unwrap()
            .text()
            .await.unwrap();

        response
    }

    pub async fn get_wcif(&self, id: &str) -> WcifResult {
        let json = self.get_wcif_api(id).await;
        parse(json)
    }

    async fn patch_wcif(&self, wcif: &Wcif, id: &str) -> String {
        let patch_url = format!("https://www.worldcubeassociation.org/api/v0/competitions/{}/wcif", id);

        let json = serde_json::to_string(wcif).unwrap();

        let response = self.client
            .patch(&patch_url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .await.unwrap()
            .text()
            .await.unwrap();

        response
    }
}

#[derive(Debug)]
pub struct WcifContainer {
    wcif: Wcif
}

impl WcifContainer {
    pub fn new(wcif: Wcif) -> WcifContainer {
        WcifContainer { wcif }
    }

    pub fn add_oauth(self, oauth: OAuth) -> WcifOAuth {
        WcifOAuth {
            cont: self,
            oauth
        }
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut Wcif {
        &mut self.wcif
    }

    pub fn get<'a>(&'a self) -> &'a Wcif {
        &self.wcif
    }

    pub async fn patch(&self, oauth: &OAuth) -> String {
        oauth.patch_wcif(&self.wcif, &self.wcif.id).await
    }

    pub fn date(&self) -> Date {
        self.wcif.schedule.start_date
    }

    pub fn events_iter(&self) -> impl Iterator<Item = &Event> {
        self.wcif.events.iter()
    }

    pub fn events_iter_mut(&mut self) -> impl Iterator<Item = &mut Event> {
        self.wcif.events.iter_mut()
    }

    pub fn patch_events(&mut self, func: impl FnMut(&mut Event)) {
        self.events_iter_mut().for_each(func);
    }

    pub fn persons_iter(&self) -> impl Iterator<Item = &Person> {
        self.wcif.persons.iter()
    }

    pub fn persons_iter_mut(&mut self) -> impl Iterator<Item = &mut Person> {
        self.wcif.persons.iter_mut()
    }

    pub fn patch_persons(&mut self, func: impl FnMut(&mut Person)) {
        self.persons_iter_mut().for_each(func);
    }

    pub fn round_iter(&self) -> impl Iterator<Item = &Round> {
        self.events_iter().flat_map(|e|e.rounds.iter())
    }

    pub fn round_iter_mut(&mut self) -> impl Iterator<Item = &mut Round> {
        self.events_iter_mut().flat_map(|e|e.rounds.iter_mut())
    }

    pub fn patch_rounds(&mut self, func: impl FnMut(&mut Round)) {
        self.round_iter_mut().for_each(func);
    }

    pub fn activity_iter(&self) -> impl Iterator<Item = &Activity> {
        self.wcif.schedule.venues.iter().flat_map(|venue|{
            venue.rooms.iter().flat_map(|room|{
                ActivityIter::new(&room.activities)
            })
        })
    }

    pub fn schedule_activity_iter(&self) -> impl Iterator<Item = &Activity> {
        self.wcif.schedule.venues.iter().flat_map(|venue|{
            venue.rooms.iter().flat_map(|room|{
                room.activities.iter()
            })
        })
    }

    pub fn overlapping_activities<'a>(&'a self) -> Vec<(&'a Activity, &'a Activity)> {
        self.schedule_activity_iter()
            .map(|act_1|{
                self.schedule_activity_iter().filter(|act_2|act_1.overlaps(act_2)).zip(std::iter::repeat(act_1))
            })
            .flatten()
            .collect()
    }

    pub fn add_groups_to_event(&mut self, event: &str, round: usize, no: usize) -> std::result::Result<&mut Vec<Activity>, ()> {
        let act = self.wcif.schedule.venues.iter_mut()
            .flat_map(|v|&mut v.rooms)
            .flat_map(|r|&mut r.activities)
            .find(|a|a.activity_code.contains(&format!("{event}-r{round}")))
            .map(|a|{
                if a.child_activities.len() != 0 {
                    return None;
                }
                a.child_activities = (0..no).map(|g|{
                    let group_time = (a.end_time - a.start_time) / no as i32;
                    let start_time = a.start_time + (group_time * g as i32);
                    let end_time = a.start_time + (group_time * (g as i32 + 1));
                    Activity { 
                        id: a.id * 1000 + g, 
                        name: format!("{}, Group {}", a.name, g + 1), 
                        activity_code: format!("{}-g{}", a.activity_code, g + 1), 
                        start_time, 
                        end_time, 
                        child_activities: vec![], 
                        scramble_set_id: None, 
                        extensions: vec![] }
                    })
                    .collect();
                Some(a)
            });
        match act {
            Some(Some(v)) => {
                Ok(&mut v.child_activities)
            }
            _ => Err(())
        }
    }
}

struct ActivityIter<'a> {
    activites: Vec<Box<dyn Iterator<Item = &'a Activity> + 'a>>
}

impl<'a> ActivityIter<'a> {
    fn new(vec: &'a Vec<Activity>) -> Self {
        ActivityIter {
            activites: vec![Box::new(vec.iter())]
        }
    }
}

impl<'a> Iterator for ActivityIter<'a> {
    type Item = &'a Activity;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = self.activites.last_mut() {
            match iter.next() {
                None => {
                    self.activites.pop();
                    self.next()
                }
                Some(v) => {
                    self.activites.push(Box::new(v.child_activities.iter()));
                    Some(v)
                }
            }
        }
        else {
            None
        }
    }
}

fn xor_password(password: &str, data: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha512::new();
    for _ in 0..100000 {
        hasher.update(password);
    }
    let f = hasher.finalize();
    let g: std::result::Result<Vec<u8>, std::io::Error> = f.bytes().collect();
    let sha_bytes = g.unwrap();
    xor_vecs(sha_bytes, data)
}


pub fn xor_vecs(mut hash: Vec<u8>, refresh: &Vec<u8>) -> Vec<u8> {
    for i in 0..64 {
        if i < refresh.len() {
            hash[i] ^= refresh[i];
        }
    }
    while hash.last() == Some(&0) {
        hash.pop();
    }
    hash
}