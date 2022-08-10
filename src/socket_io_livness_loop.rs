use std::{sync::Arc, time::Duration};

use crate::{
    MySocketIoConnection, MySocketIoConnectionsCallbacks, MySocketIoMessage, SocketIoList,
};

pub async fn start(
    connect_events: Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    sockets_list: Arc<SocketIoList>,
    my_socket_io_connection: Arc<MySocketIoConnection>,
    ping_timeout: Duration,
) {
    println!(
        "Socket.IO {} started livness loop",
        my_socket_io_connection.id
    );

    while my_socket_io_connection.is_connected() {
        if my_socket_io_connection.in_web_socket_model() {
            my_socket_io_connection
                .send_message(&MySocketIoMessage::Ping)
                .await;
        }

        tokio::time::sleep(ping_timeout).await;
    }

    crate::process_disconnect(&sockets_list, &my_socket_io_connection, &connect_events).await;
}
