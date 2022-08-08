pub enum MySocketIoMessage {
    Ping,
    Pong,
    Disconnect,
    Message(String),
}

impl MySocketIoMessage {
    pub fn to_string(&self) -> String {
        match self {
            MySocketIoMessage::Ping => "2".to_string(),
            MySocketIoMessage::Pong => "3".to_string(),
            MySocketIoMessage::Disconnect => "41".to_string(),
            MySocketIoMessage::Message(msg) => format!("42{}", msg),
        }
    }
}
