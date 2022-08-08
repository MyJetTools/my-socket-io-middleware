use std::{sync::Arc, time::Duration};

use hyper_tungstenite::tungstenite::Message;
use my_http_server_web_sockets::MyWebSocket;

use crate::MySocketIoMessage;

pub async fn start(socket: Arc<MyWebSocket>) {
    println!("Socket {} started ping loop", socket.id);
    while socket.is_connected() {
        socket
            .send_message(Message::Text(MySocketIoMessage::Ping.to_string()))
            .await;

        socket
            .send_message(Message::Text(
                MySocketIoMessage::Message("[\"test\", \"test2\"]".to_string()).to_string(),
            ))
            .await;

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    println!("Socket {} exited ping loop", socket.id);
}
