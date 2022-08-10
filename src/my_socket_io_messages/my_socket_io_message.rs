use super::MySocketIoTextPayload;

pub enum MySocketIoMessage {
    Ping,
    Pong,
    Disconnect,
    Message(MySocketIoTextPayload),
    Ack(MySocketIoTextPayload),
}

impl MySocketIoMessage {
    pub fn to_string(&self) -> String {
        match self {
            MySocketIoMessage::Ping => "2".to_string(),
            MySocketIoMessage::Pong => "3".to_string(),
            MySocketIoMessage::Disconnect => "41".to_string(),
            MySocketIoMessage::Message(msg) => {
                let mut result = Vec::new();
                result.extend_from_slice("42".as_bytes());
                msg.serialize(&mut result);
                String::from_utf8(result).unwrap()
            }
            MySocketIoMessage::Ack(msg) => {
                let mut result = Vec::new();
                result.extend_from_slice("43".as_bytes());
                msg.serialize(&mut result);
                String::from_utf8(result).unwrap()
            }
        }
    }

    pub fn parse(str: &str) -> Option<Self> {
        if str.starts_with("42") {
            return Some(Self::Message(MySocketIoTextPayload::parse(str.as_bytes())));
        }

        if str.starts_with("43") {
            return Some(Self::Message(MySocketIoTextPayload::parse(str.as_bytes())));
        }

        None
    }
}

#[cfg(test)]
mod test {
    use my_json::json_writer::JsonArrayWriter;

    use crate::my_socket_io_messages::*;

    #[test]
    fn test_message() {
        let mut json_writer = JsonArrayWriter::new();

        json_writer.write_string_element("Test1");
        json_writer.write_string_element("Test2");

        let msg = MySocketIoMessage::Message(MySocketIoTextPayload {
            nsp: None,
            data: String::from_utf8(json_writer.build()).unwrap(),
            id: None,
        });
        assert_eq!(msg.to_string(), "42[\"Test1\",\"Test2\"]");
    }

    #[test]
    fn test_parse_message() {
        let src = "420[\"chat message\",\"123\",{\"name\":\"chat\"}]";
        let message = MySocketIoMessage::parse(src).unwrap();

        if let MySocketIoMessage::Message(message) = message {
            assert_eq!(message.nsp.is_none(), true);
            assert_eq!(message.id.as_ref().unwrap(), "0");
            assert_eq!(message.data.as_str(), &src[3..]);
        } else {
            panic!("Should not be here");
        }
    }
}
