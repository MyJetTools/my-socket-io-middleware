use std::sync::{atomic::AtomicBool, Arc};

use hyper_tungstenite::tungstenite::Message;
use my_http_server_web_sockets::MyWebSocket;
use rust_extensions::{
    date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds},
    TaskCompletion,
};
use tokio::sync::Mutex;

use socket_io_utils::my_socket_io_messages::*;

pub struct MySocketIoSingleThreaded {
    web_socket: Option<Arc<MyWebSocket>>,
    long_pooling: Option<TaskCompletion<String, String>>,
    updgraded_to_websocket: bool,
}

pub struct MySocketIoConnection {
    single_threaded: Mutex<MySocketIoSingleThreaded>,
    pub id: String,
    pub created: DateTimeAsMicroseconds,
    pub last_incoming_moment: AtomicDateTimeAsMicroseconds,
    connected: AtomicBool,
    has_web_socket: AtomicBool,
}

impl MySocketIoConnection {
    pub fn new(id: String, web_socket: Option<Arc<MyWebSocket>>) -> Self {
        let has_web_socket = web_socket.is_some();
        Self {
            single_threaded: Mutex::new(MySocketIoSingleThreaded {
                web_socket,
                long_pooling: None,
                updgraded_to_websocket: false,
            }),
            id,
            created: DateTimeAsMicroseconds::now(),
            last_incoming_moment: AtomicDateTimeAsMicroseconds::now(),
            connected: AtomicBool::new(true),
            has_web_socket: AtomicBool::new(has_web_socket),
        }
    }

    pub async fn upgrade_to_websocket(&self) {
        let mut write_access = self.single_threaded.lock().await;

        write_access.updgraded_to_websocket = true;
        if let Some(mut removed) = write_access.long_pooling.take() {
            removed.set_error(MySocketIoMessage::Disconnect.as_str().to_string());
        }

        self.has_web_socket
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn in_web_socket_model(&self) -> bool {
        self.has_web_socket
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn has_web_socket(&self, web_socket_id: i64) -> bool {
        let read_access = self.single_threaded.lock().await;
        if let Some(web_socket) = &read_access.web_socket {
            return web_socket.id == web_socket_id;
        }

        false
    }

    pub async fn get_web_socket(&self) -> Option<Arc<MyWebSocket>> {
        let read_access = self.single_threaded.lock().await;
        read_access.web_socket.clone()
    }

    pub fn update_incoming_activity(&self) {
        self.last_incoming_moment
            .update(DateTimeAsMicroseconds::now());
    }

    pub async fn send_message(&self, message: &MySocketIoMessage) {
        let web_socket = {
            let read_access = self.single_threaded.lock().await;
            read_access.web_socket.clone()
        };

        if let Some(web_socket) = web_socket {
            web_socket
                .send_message(Message::Text(message.as_str().to_string()))
                .await;
        }
    }

    pub async fn add_web_socket(&self, web_socket: Arc<MyWebSocket>) {
        let new_id = web_socket.id;
        let mut write_access = self.single_threaded.lock().await;

        if let Some(old_websocket) = write_access.web_socket.replace(web_socket) {
            old_websocket
                .send_message(hyper_tungstenite::tungstenite::Message::Text(format!(
                    "SocketIO WebSocket {} has been kicked by Websocket {} ",
                    old_websocket.id, new_id
                )))
                .await;
        }
    }

    pub async fn disconnect(&self) -> Option<Arc<MyWebSocket>> {
        let mut write_access = self.single_threaded.lock().await;

        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let mut result = None;

        if let Some(web_socket) = write_access.web_socket.take() {
            web_socket.disconnect().await;
            result = Some(web_socket);
        }

        if let Some(mut long_pooling) = write_access.long_pooling.take() {
            long_pooling.set_error(format!("Canceling this LongPool since we disconnect it."));
        }

        result
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}
