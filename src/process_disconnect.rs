use std::sync::Arc;

use crate::{MySocketIoConnection, MySocketIoConnectionsCallbacks, SocketIoList};

pub async fn process_disconnect(
    sockets_list: &Arc<SocketIoList>,
    socket_io_connection: &Arc<MySocketIoConnection>,
    connect_events: &Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
) {
    let removed_connection = sockets_list.remove(socket_io_connection.id.as_str()).await;

    if let Some(removed_connection) = removed_connection {
        println!("Socket.IO {} is diconnectd", removed_connection.id);
        connect_events.disconnected(removed_connection).await;
    }
}
