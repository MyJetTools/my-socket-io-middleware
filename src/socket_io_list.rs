use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::MySocketIoConnection;

struct SocketIdListInner {
    sockets_by_web_socket_id: HashMap<i64, Arc<MySocketIoConnection>>,
    sockets_by_my_socket_io_id: HashMap<String, Arc<MySocketIoConnection>>,
}

pub struct SocketIoList {
    sockets: RwLock<SocketIdListInner>,
}

impl SocketIoList {
    pub fn new() -> Self {
        Self {
            sockets: RwLock::new(SocketIdListInner {
                sockets_by_web_socket_id: HashMap::new(),
                sockets_by_my_socket_io_id: HashMap::new(),
            }),
        }
    }

    pub async fn add_socket_io(&self, socket_io_connection: Arc<MySocketIoConnection>) {
        let mut write_access = self.sockets.write().await;
        write_access.sockets_by_my_socket_io_id.insert(
            socket_io_connection.id.clone(),
            socket_io_connection.clone(),
        );
    }

    pub async fn get_by_socket_io_id(
        &self,
        socket_io_id: &str,
    ) -> Option<Arc<MySocketIoConnection>> {
        let read_access = self.sockets.read().await;
        let result = read_access.sockets_by_my_socket_io_id.get(socket_io_id)?;
        Some(result.clone())
    }

    pub async fn get_by_web_socket_id(
        &self,
        web_socket_io: i64,
    ) -> Option<Arc<MySocketIoConnection>> {
        let read_access = self.sockets.read().await;
        let result = read_access.sockets_by_web_socket_id.get(&web_socket_io)?;
        Some(result.clone())
    }
}
