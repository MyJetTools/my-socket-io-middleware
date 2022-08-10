pub struct MySocketIoTextMessage {
    pub nsp: Option<String>,
    pub data: String,
    pub id: Option<String>,
}

impl MySocketIoTextMessage {
    pub fn parse(src: &[u8]) -> Self {
        let open_array = find(src, 2, b'[');
        Self {
            nsp: extract_nsp(src),
            data: String::from_utf8(src[open_array..].to_vec()).unwrap(),
            id: extract_ack_id(src),
        }
    }
    pub fn serialize(&self, dest: &mut Vec<u8>) {
        if let Some(nsp) = &self.nsp {
            dest.extend_from_slice(nsp.as_bytes());
            dest.push(',' as u8);
        }

        if let Some(id) = &self.id {
            dest.extend_from_slice(id.as_bytes());
        }

        dest.extend_from_slice(self.data.as_bytes());
    }
}

pub enum MySocketIoMessage {
    Ping,
    Pong,
    Disconnect,
    Message(MySocketIoTextMessage),
    Ack(MySocketIoTextMessage),
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

    pub fn parse(str: &str) -> Self {
        if str.starts_with("42") {
            return Self::Message(MySocketIoTextMessage::parse(str.as_bytes()));
        }

        if str.starts_with("43") {
            return Self::Message(MySocketIoTextMessage::parse(str.as_bytes()));
        }

        panic!("Can not parse message: {}", str)
    }
}

fn find(raw: &[u8], start_pos: usize, find_element: u8) -> usize {
    for pos in start_pos..raw.len() {
        if raw[pos] == find_element {
            return pos;
        }
    }

    panic!("Can not find open array");
}

fn extract_nsp(raw: &[u8]) -> Option<String> {
    if raw[3] != b'/' {
        return None;
    }
    let end = find(raw, 4, b',');

    Some(String::from_utf8(raw[4..end].to_vec()).unwrap())
}

fn extract_ack_id(raw: &[u8]) -> Option<String> {
    if !is_number(raw[2]) {
        return None;
    }

    for i in 2..raw.len() {
        if !is_number(raw[i]) {
            return Some(String::from_utf8(raw[2..i].to_vec()).unwrap());
        }
    }

    panic!(
        "Can not extract ack id. Message: {}",
        String::from_utf8(raw.to_vec()).unwrap()
    );
}

fn is_number(c: u8) -> bool {
    c >= b'0' && c <= b'9'
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
            data: String::from_utf8(json_writer.build()).unwrap(),
            id: None,
        });
        assert_eq!(msg.to_string(), "42[\"Test1\",\"Test2\"]");
    }

    #[test]
    fn test_parse_message() {
        let src = "420[\"chat message\",\"123\",{\"name\":\"chat\"}]";
        let message = MySocketIoMessage::parse(src);

        if let MySocketIoMessage::Message(message) = message {
            assert_eq!(message.nsp.is_none(), true);
            assert_eq!(message.id.as_ref().unwrap(), "0");
            assert_eq!(message.data.as_str(), &src[3..]);
        } else {
            panic!("Should not be here");
        }
    }
}
