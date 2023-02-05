use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, MutexGuard};

use wca_oauth::WcifContainer;

#[derive(Clone)]
pub struct DB {
    inner: Arc<Mutex<HashMap<(String, String), WcifContainer>>>
}

pub struct DBLock<'a> {
    inner: MutexGuard<'a, HashMap<(String, String), WcifContainer>>,
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
        self.inner.lock()
            .await
            .insert((competition, auth_code), wcif);
    }

    pub async fn garbage_elimination(&mut self) {
        println!("Garbage eliminated!");
    }
}

impl DBLock<'_> {
    pub fn get(&mut self) -> Option<&mut WcifContainer> {
        self.inner.get_mut(&self.key)
    }
}
