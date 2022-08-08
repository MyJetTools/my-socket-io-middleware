pub fn compile_negotiate_response(sid: &str) -> String {
    let mut result = Vec::new();

    result.extend_from_slice("0{\"sid\":\"".as_bytes());

    result.extend_from_slice(sid.as_bytes());

    result.extend_from_slice("\",\"upgrades\":[\"websocket\"],\"pingInterval\":25000,\"pingTimeout\":5000,\"maxPayload\":1000000}".as_bytes());

    String::from_utf8(result).unwrap()
}
