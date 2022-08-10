use std::sync::Arc;

use my_http_server_web_sockets::MyWebSocket;

use crate::{MySocketIoConnection, MySocketIoConnectionsCallbacks, SocketIoList, SocketIoSettings};

pub async fn process_connect(
    connections_callback: &Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    socket_io_list: &Arc<SocketIoList>,
    settings: &Arc<SocketIoSettings>,
    web_socket: Option<Arc<MyWebSocket>>,
) -> String {
    let sid = uuid::Uuid::new_v4().to_string();

    let sid = sid.replace("-", "")[..8].to_string();

    let result = crate::my_socket_io_messages::compile_negotiate_response(sid.as_str(), settings);

    let socket_io = MySocketIoConnection::new(sid, web_socket);
    let socket_io_connection = Arc::new(socket_io);

    connections_callback
        .connected(socket_io_connection.clone())
        .await
        .unwrap();

    socket_io_list.add_socket_io(socket_io_connection).await;

    result
}
