use std::{collections::HashMap, sync::Arc, time::Instant};

use tokio::sync::{Mutex, MutexGuard};

use wca_oauth::WcifContainer;

#[derive(Clone)]
pub struct DB {
    inner: Arc<Mutex<HashMap<(String, String), (WcifContainer, Instant)>>>
}

pub struct DBLock<'a> {
    inner: MutexGuard<'a, HashMap<(String, String), (WcifContainer, Instant)>>,
    key: (String, String),
}

impl DB {
    pub fn new() -> DB {
        DB { inner: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub async fn get_wcif_lock(&mut self, competition: String, auth_code: String) -> DBLock {
        DBLock { inner: self.inner.lock().await, key: (competition, auth_code) }
    }

    pub async fn insert_wcif(&mut self, competition: String, auth_code: String, wcif: WcifContainer) {
        let str = format!("wcif stored for: {} {}", auth_code, competition);
        dbg!(str);
        self.inner.lock()
            .await
            .insert((competition, auth_code), (wcif, std::time::Instant::now()));
    }

    pub async fn garbage_elimination(&mut self) {
        let now = std::time::Instant::now();
        self.inner
            .lock()
            .await
            .retain(|_, (_, time)| {
                let time_since = now.duration_since(*time);
                time_since > std::time::Duration::from_secs(1800)
            });
    }
}

impl DBLock<'_> {
    pub fn get(&mut self) -> Option<&mut WcifContainer> {
        let str = format!("wcif retrieved for: {} {}", self.key.0, self.key.1);
        dbg!(str);
        self.inner.get_mut(&self.key).map(|s| &mut s.0)
    }
}
