use std::{sync::Arc, time::Duration};

use hyper_tungstenite::tungstenite::Message;
use my_http_server::HttpFailResult;
use my_http_server_web_sockets::{MyWebSocket, WebSocketMessage};
use my_json::json_reader::array_parser::ArrayToJsonObjectsSplitter;
use socket_io_utils::SocketIoSettings;

use crate::{
    namespaces::SocketIoNameSpaces, MySocketIoConnection, MySocketIoConnectionsCallbacks,
    SocketIoList,
};

use socket_io_utils::{
    my_socket_io_messages::MySocketIoMessage,
    my_socket_io_messages::{GrandAccessData, MySocketIoTextPayload},
};

const DEFAULT_NAMESPACE: &str = "/";

fn get_nsp(value: &Option<String>) -> &str {
    if let Some(nsp) = &value {
        nsp
    } else {
        DEFAULT_NAMESPACE
    }
}

pub struct WebSocketCallbacks {
    pub socket_io_list: Arc<SocketIoList>,
    pub registered_sockets: Arc<SocketIoNameSpaces>,
    pub connections_callback: Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    pub settings: Arc<SocketIoSettings>,
}

impl WebSocketCallbacks {
    async fn callback_message(
        &self,
        socket_io: &Arc<MySocketIoConnection>,
        msg: MySocketIoTextPayload,
    ) {
        let nsp_str = get_nsp(&msg.nsp);

        if let Some(socket) = self.registered_sockets.get(nsp_str).await {
            let mut event_name = None;
            let mut event_data = None;

            let mut i = 0;
            for data in msg.data.as_bytes().split_array_json_to_objects() {
                let data = data.unwrap();
                if i == 0 {
                    event_name = Some(std::str::from_utf8(data).unwrap());
                } else if i == 1 {
                    event_data = Some(std::str::from_utf8(data).unwrap());
                } else {
                    break;
                }

                i += 1;
            }

            let event_name = event_name.unwrap();
            let event_data = event_data.unwrap();

            if let Some(ack_data) = socket.on(&event_name, &event_data).await {
                let ack_contract = MySocketIoMessage::Ack(MySocketIoTextPayload {
                    nsp: msg.nsp,
                    data: ack_data,
                    id: msg.id,
                });

                socket_io.send_message(&ack_contract).await;
            }
        }
    }
}

#[async_trait::async_trait]
impl my_http_server_web_sockets::MyWebSocketCallback for WebSocketCallbacks {
    async fn connected(
        &self,
        my_web_socket: Arc<MyWebSocket>,
        disconnect_timeout: Duration,
    ) -> Result<(), HttpFailResult> {
        #[cfg(feature = "debug_ws")]
        println!("connected web_socket:{}", my_web_socket.id);

        if let Some(query_string) = my_web_socket.get_query_string() {
            let sid = query_string.get_optional("sid");

            if sid.is_none() {
                let (socket_io, response) = crate::process_connect(
                    &self.connections_callback,
                    &self.socket_io_list,
                    &self.settings,
                    Some(my_web_socket.clone()),
                )
                .await;

                my_web_socket.send_message(Message::Text(response)).await;

                tokio::spawn(super::socket_io_livness_loop::start(
                    self.connections_callback.clone(),
                    self.socket_io_list.clone(),
                    socket_io,
                    self.settings.get_ping_timeout(),
                    self.settings.get_ping_interval(),
                ));
                return Ok(());
            }

            let sid = sid.unwrap();

            match self
                .socket_io_list
                .assign_web_socket_to_socket_io(sid.value, my_web_socket.clone())
                .await
            {
                Some(socket_io) => {
                    tokio::spawn(super::socket_io_livness_loop::start(
                        self.connections_callback.clone(),
                        self.socket_io_list.clone(),
                        socket_io,
                        self.settings.get_ping_timeout(),
                        self.settings.get_ping_interval(),
                    ));
                }
                None => {
                    my_web_socket
                        .send_message(Message::Text(format!(
                            "Socket.IO with id {} is not found",
                            sid.value,
                        )))
                        .await;

                    return Ok(());
                }
            };
        }

        Ok(())
    }

    async fn disconnected(&self, my_web_socket: Arc<MyWebSocket>) {
        #[cfg(feature = "debug_ws")]
        println!("disconnected web_socket:{}", my_web_socket.id);
        let find_result = self
            .socket_io_list
            .get_by_web_socket_id(my_web_socket.id)
            .await;

        if let Some(socket_io) = find_result {
            crate::process_disconnect(&self.socket_io_list, &socket_io, &self.connections_callback)
                .await;
        }
    }
    async fn on_message(&self, my_web_socket: Arc<MyWebSocket>, message: WebSocketMessage) {
        #[cfg(feature = "debug_ws")]
        println!("Websocket{}, MSG: {:?}", my_web_socket.id, message);

        let socket_io = self
            .socket_io_list
            .get_by_web_socket_id(my_web_socket.id)
            .await;

        if let Some(socket_io_ref) = socket_io.as_ref() {
            socket_io_ref.update_incoming_activity();
        }

        if let WebSocketMessage::String(value) = &message {
            if value == socket_io_utils::my_socket_io_messages::ENGINE_IO_PING_PROBE_PAYLOAD {
                my_web_socket
                    .send_message(Message::Text(
                        socket_io_utils::my_socket_io_messages::ENGINE_IO_PONG_PROBE_PAYLOAD
                            .to_string(),
                    ))
                    .await;
                return;
            }

            if value == socket_io_utils::my_socket_io_messages::ENGINE_IO_UPGRADE_PAYLOAD {
                if let Some(socket_io) = socket_io.as_ref() {
                    socket_io.upgrade_to_websocket().await;
                } else {
                    my_web_socket
                        .send_message(Message::Text("SocketIo not found".to_string()))
                        .await;
                    my_web_socket.disconnect().await;
                }
                return;
            }

            if let Some(message) = MySocketIoMessage::parse(value.as_str()) {
                match message {
                    MySocketIoMessage::Message(message) => {
                        if let Some(socket_io_connection) = socket_io.as_ref() {
                            self.callback_message(socket_io_connection, message).await;
                        }
                    }
                    MySocketIoMessage::RequestAccess(nsp) => {
                        if let Some(socket_io_connection) = socket_io.as_ref() {
                            let nsp_str = get_nsp(&nsp);

                            if self.registered_sockets.has_nsp(nsp_str).await {
                                let granted_message =
                                    MySocketIoMessage::GrandAccess(GrandAccessData {
                                        nsp,
                                        sid: socket_io_connection.id.clone(),
                                    });

                                socket_io_connection.send_message(&granted_message).await;
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}
