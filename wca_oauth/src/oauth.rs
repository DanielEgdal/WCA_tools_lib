use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;
use crate::*;
use crate::Competition;

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
    pub async fn get_auth(client_id: String, client_secret: String, redirect_uri: String, auth_code: String) -> Self {
        let mut oauth = Self {
            access_token: format!(""),
            refresh_token: format!(""),
            client_id,
            client_secret,
            redirect_uri,
            client: reqwest::Client::new()
        };
        oauth.get_auth_explicit_flow(auth_code).await;
        oauth
    }

    /// If you use this you need to get a token before hand. Refresh cannot be done with this type and will crash.
    pub async fn get_auth_implicit(client_id: String, access_token: String, redirect_uri: String) -> Self {
        Self {
            access_token,
            refresh_token: String::new(),
            client_id,
            client_secret: String::new(),
            redirect_uri,
            client: reqwest::Client::new()
        }
    }


    async fn get_auth_explicit_flow(&mut self, code: String) {
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

    pub async fn refresh_auth(&mut self) {
        let mut params = HashMap::new();

        params.insert("grant_type", "refresh_token");
        params.insert("client_id", &self.client_id);
        params.insert("client_secret", &self.client_secret);
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

    pub async fn get_competitions_managed_by_me(&self) -> Vec<Competition> {
        let url = "https://www.worldcubeassociation.org/api/v0/competitions?managed_by_me=true";

        let json = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await.unwrap()
            .text()
            .await.unwrap();
        
        Competition::from_json(&json)
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
    pub(crate) wcif: Wcif
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
