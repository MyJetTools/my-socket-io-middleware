use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::MySocketIo;

pub struct SocketIoNameSpaces {
    items: Mutex<HashMap<String, Arc<dyn MySocketIo + Send + Sync + 'static>>>,
}

impl SocketIoNameSpaces {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get(&self, nsp: &str) -> Option<Arc<dyn MySocketIo + Send + Sync + 'static>> {
        let read_access = self.items.lock().await;
        read_access.get(nsp).cloned()
    }

    pub async fn add(&self, socket: Arc<dyn MySocketIo + Send + Sync + 'static>) {
        let mut write_access = self.items.lock().await;
        write_access.insert(socket.get_nsp().to_string(), socket);
    }

    pub async fn has_nsp(&self, nsp: &str) -> bool {
        let read_access = self.items.lock().await;
        return read_access.contains_key(nsp);
    }
}
