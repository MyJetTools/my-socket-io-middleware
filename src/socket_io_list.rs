use std::{collections::HashMap, sync::Arc};

use my_http_server_web_sockets::MyWebSocket;
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
        let web_socket = socket_io_connection.get_web_socket().await;
        let mut write_access = self.sockets.write().await;
        write_access.sockets_by_my_socket_io_id.insert(
            socket_io_connection.id.clone(),
            socket_io_connection.clone(),
        );

        if let Some(web_socket) = web_socket {
            write_access
                .sockets_by_web_socket_id
                .insert(web_socket.id, socket_io_connection.clone());
        }
    }

    pub async fn assign_web_socket_to_socket_io(
        &self,
        socket_io_id: &str,
        web_socket: Arc<MyWebSocket>,
    ) -> Option<Arc<MySocketIoConnection>> {
        let found = {
            let mut write_access = self.sockets.write().await;

            let found = {
                if let Some(found) = write_access.sockets_by_my_socket_io_id.get(socket_io_id) {
                    Some(found.clone())
                } else {
                    None
                }
            };

            if let Some(found) = found {
                write_access
                    .sockets_by_web_socket_id
                    .insert(web_socket.id, found.clone());
                Some(found)
            } else {
                None
            }
        };

        if let Some(found) = found {
            found.add_web_socket(web_socket).await;
            Some(found)
        } else {
            None
        }
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

    pub async fn remove(&self, socket_io_id: &str) -> Option<Arc<MySocketIoConnection>> {
        let removed_socket_io = {
            let mut write_access = self.sockets.write().await;
            write_access.sockets_by_my_socket_io_id.remove(socket_io_id)
        };

        if let Some(removed_socket_io) = &removed_socket_io {
            let web_socket = removed_socket_io.disconnect().await;
            if let Some(web_socket) = web_socket {
                let mut write_access = self.sockets.write().await;
                write_access.sockets_by_web_socket_id.remove(&web_socket.id);
            }
        } else {
            return None;
        }

        removed_socket_io
    }
}
