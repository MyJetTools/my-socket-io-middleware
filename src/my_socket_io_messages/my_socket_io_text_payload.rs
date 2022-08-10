pub struct MySocketIoTextPayload {
    pub nsp: Option<String>,
    pub data: String,
    pub id: Option<String>,
}

impl MySocketIoTextPayload {
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
