use std::sync::Arc;

use hyper_tungstenite::tungstenite::Message;
use my_http_server::HttpFailResult;
use my_http_server_web_sockets::{MyWebSocket, WebSocketMessage};

use crate::SocketIoList;

pub struct WebSocketCallbacks<TCustomData: Send + Sync + 'static> {
    pub socket_io_list: Arc<dyn SocketIoList<TCustomData> + Send + Sync + 'static>,
}

#[async_trait::async_trait]
impl<TCustomData: Send + Sync + 'static> my_http_server_web_sockets::MyWebSockeCallback
    for WebSocketCallbacks<TCustomData>
{
    async fn connected(&self, my_web_socket: Arc<MyWebSocket>) -> Result<(), HttpFailResult> {
        println!("connected web_socket:{}", my_web_socket.id);

        if let Some(query_string) = my_web_socket.get_query_string() {
            let sid = query_string.get_required("sid")?;

            match self.socket_io_list.get(sid.value).await {
                Some(socket_io) => {
                    socket_io.add_web_socket(my_web_socket).await;
                }
                None => {
                    my_web_socket
                        .send_message(Message::Text(
                            format!("SocketIo not found with id {}", sid.value).to_string(),
                        ))
                        .await;
                }
            };
        }

        Ok(())
    }

    async fn disconnected(&self, my_web_socket: Arc<MyWebSocket>) {
        println!("disconnected web_socket:{}", my_web_socket.id);
        let find_result = self
            .socket_io_list
            .get_by_web_socket_id(my_web_socket.id)
            .await;

        if let Some(socket_io) = find_result {
            socket_io.disconnect().await;
        }
    }
    async fn on_message(&self, my_web_socket: Arc<MyWebSocket>, message: WebSocketMessage) {
        println!("Websocket{}, MSG: {:?}", my_web_socket.id, message);

        if let WebSocketMessage::String(value) = &message {
            if value == "2probe" {
                my_web_socket
                    .send_message(Message::Text("3probe".to_string()))
                    .await;
                return;
            }

            if value == "5" {
                tokio::spawn(super::socket_io_ping_loop::start(my_web_socket.clone()));
                if let Some(socket_io) = self
                    .socket_io_list
                    .get_by_web_socket_id(my_web_socket.id)
                    .await
                {
                    socket_io.upgrade_to_websocket().await;
                }
            }
        }
    }
}
