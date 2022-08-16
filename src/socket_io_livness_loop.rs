use std::{sync::Arc, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{MySocketIoConnection, MySocketIoConnectionsCallbacks, SocketIoList};

use socket_io_utils::my_socket_io_messages::MySocketIoMessage;

pub async fn start(
    connect_events: Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    sockets_list: Arc<SocketIoList>,
    my_socket_io_connection: Arc<MySocketIoConnection>,
    ping_timeout: Duration,
    ping_disconnect: Duration,
) {
    println!(
        "Socket.IO {} started livness loop",
        my_socket_io_connection.id
    );

    while my_socket_io_connection.is_connected() {
        let now = DateTimeAsMicroseconds::now();

        let last_incoming_moment = my_socket_io_connection.last_incoming_moment.as_date_time();

        let duration = now.duration_since(last_incoming_moment);

        if duration.as_positive_or_zero() >= ping_disconnect {
            println!(
                "Socket.IO {} disconnected because of ping timeout",
                my_socket_io_connection.id
            );
            break;
        }

        if my_socket_io_connection.in_web_socket_model() {
            my_socket_io_connection
                .send_message(&MySocketIoMessage::Ping)
                .await;
        }

        tokio::time::sleep(ping_timeout).await;
    }

    crate::process_disconnect(&sockets_list, &my_socket_io_connection, &connect_events).await;
}
