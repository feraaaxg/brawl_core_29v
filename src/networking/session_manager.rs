use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;
use crate::networking::client::ClientConnection;

pub struct SessionManager {
    clients: HashMap<usize, Arc<Mutex<ClientConnection>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn new_session(&mut self, session: Arc<Mutex<ClientConnection>>) {
        let id = self.get_last_session().map(|id| id + 1).unwrap_or(0);
        self.clients.insert(id, session);
    }

    pub fn remove_session(&mut self, id: usize) {
        self.clients.remove(&id);
    }

    pub fn get_last_session(&self) -> Option<usize> {
        self.clients.keys().last().cloned()
    }

    pub fn get_client(&mut self, id: usize) -> Option<&Arc<Mutex<ClientConnection>>> {
        self.clients.get(&id)
    }
}