use my_json::json_writer::JsonArrayWriter;

pub enum MySocketIoTextData {
    Raw(String),
    Json(JsonArrayWriter),
}

pub struct MySocketIoTextMessage {
    pub nsp: Option<String>,
    pub data: MySocketIoTextData,
    pub id: Option<String>,
}

pub enum MySocketIoMessage {
    Ping,
    Pong,
    Disconnect,
    Message(MySocketIoTextMessage),
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

                if let Some(nsp) = &msg.nsp {
                    result.extend_from_slice(nsp.as_bytes());
                    result.push(',' as u8);
                }

                if let Some(id) = &msg.id {
                    result.extend_from_slice(id.as_bytes());
                }

                match &msg.data {
                    MySocketIoTextData::Raw(raw) => result.extend_from_slice(raw.as_bytes()),
                    MySocketIoTextData::Json(json_writer) => {
                        json_writer.build_into(&mut result);
                    }
                };
                String::from_utf8(result).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use my_json::json_writer::JsonArrayWriter;

    use crate::{my_socket_io_message::MySocketIoTextMessage, *};

    #[test]
    fn test_message() {
        let mut json_writer = JsonArrayWriter::new();

        json_writer.write_string_element("Test1");
        json_writer.write_string_element("Test2");

        let msg = MySocketIoMessage::Message(MySocketIoTextMessage {
            nsp: None,
            data: MySocketIoTextData::Json(json_writer),
            id: None,
        });
        assert_eq!(msg.to_string(), "42[\"Test1\",\"Test2\"]");
    }
}
