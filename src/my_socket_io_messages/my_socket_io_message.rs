use rust_extensions::StrOrString;

use super::MySocketIoTextPayload;

pub enum MySocketIoMessage {
    Ping,
    Pong,
    Disconnect,
    Message(MySocketIoTextPayload),
    Ack(MySocketIoTextPayload),
    Connect(Option<String>),
}

impl MySocketIoMessage {
    pub fn as_str(&self) -> StrOrString {
        match self {
            MySocketIoMessage::Ping => StrOrString::crate_as_str("2"),
            MySocketIoMessage::Pong => StrOrString::crate_as_str("3"),
            MySocketIoMessage::Disconnect => StrOrString::crate_as_str("41"),
            MySocketIoMessage::Message(msg) => {
                let mut result = Vec::new();
                result.extend_from_slice("42".as_bytes());
                msg.serialize(&mut result);
                StrOrString::crate_as_string(String::from_utf8(result).unwrap())
            }
            MySocketIoMessage::Ack(msg) => {
                let mut result = Vec::new();
                result.extend_from_slice("43".as_bytes());
                msg.serialize(&mut result);
                StrOrString::crate_as_string(String::from_utf8(result).unwrap())
            }
            MySocketIoMessage::Connect(nsp) => {
                if let Some(nsp) = nsp {
                    let mut result = Vec::new();
                    result.extend_from_slice("40".as_bytes());
                    result.extend_from_slice(nsp.as_bytes());
                    result.push(b',');
                    StrOrString::crate_as_string(String::from_utf8(result).unwrap())
                } else {
                    StrOrString::crate_as_str("40")
                }
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

        if str.starts_with("40") {
            if str == "40" {
                return Some(Self::Connect(None));
            }

            return Some(Self::Connect(Some(str[2..str.len() - 1].to_string())));
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
        assert_eq!(msg.as_str().as_str(), "42[\"Test1\",\"Test2\"]");
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

    #[test]
    fn test_connect_message() {
        let connect_message = MySocketIoMessage::Connect(None);
        let result = connect_message.as_str();
        assert_eq!("40", result.as_str());
        if let MySocketIoMessage::Connect(result) =
            MySocketIoMessage::parse(result.as_str()).unwrap()
        {
            assert_eq!(result.is_none(), true);
        } else {
            panic!("Should not be here");
        }

        let connect_message = MySocketIoMessage::Connect(Some("/admin".to_string()));

        let result = connect_message.as_str();

        assert_eq!("40/admin,", result.as_str());

        if let MySocketIoMessage::Connect(result) =
            MySocketIoMessage::parse(result.as_str()).unwrap()
        {
            if let Some(nsp) = result {
                assert_eq!("/admin", nsp);
            }
        } else {
            panic!("Should not be here");
        }
    }
}
