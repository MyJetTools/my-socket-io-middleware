use std::time::Duration;

use crate::SocketIoSettings;

pub fn compile_connect_payload(sid: &str) -> Vec<u8> {
    let mut content = Vec::new();
    content.extend_from_slice("40{\"sid\":\"".as_bytes());
    content.extend_from_slice(sid.as_bytes());
    content.extend_from_slice("\"}".as_bytes());

    content
}

pub fn compile_negotiate_response(sid: &str, socket_io_settings: &SocketIoSettings) -> String {
    let mut result = Vec::new();

    result.extend_from_slice("0{\"sid\":\"".as_bytes());

    result.extend_from_slice(sid.as_bytes());

    result.extend_from_slice("\",\"upgrades\":[\"websocket\"],\"pingInterval\":".as_bytes());
    write_duration(&mut result, socket_io_settings.get_ping_interval());

    result.extend_from_slice(",\"pingTimeout\":".as_bytes());

    write_duration(&mut result, socket_io_settings.get_ping_timeout());

    result.extend_from_slice(",\"maxPayload\":".as_bytes());

    result.extend_from_slice(
        socket_io_settings
            .get_max_payload_size()
            .to_string()
            .as_bytes(),
    );

    result.extend_from_slice("}".as_bytes());

    String::from_utf8(result).unwrap()
}

fn write_duration(dest: &mut Vec<u8>, duration: Duration) {
    let millis = duration.as_millis();
    let millis = millis.to_string();
    dest.extend_from_slice(millis.as_bytes());
}
