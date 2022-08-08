use my_http_server::{HttpFailResult, HttpOkResult, HttpOutput, WebContentType};

use crate::app::AppContext;

pub async fn handle_long_pool(app: &AppContext, sid: &str) -> Result<HttpOkResult, HttpFailResult> {
    let web_socket = app.socket_io_list.get(sid).await;

    if web_socket.is_none() {
        return HttpOutput::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: format!("SocketId with id {} is not founnd", sid).into_bytes(),
        }
        .into_ok_result(true)
        .into();
    }

    let web_socket = web_socket.unwrap();

    let awaiter = web_socket.set_long_pooling_task().await;

    let result = awaiter.get_result().await;

    match result {
        Ok(result) => HttpOutput::as_text(result).into_ok_result(false).into(),
        Err(err) => Err(HttpFailResult {
            write_telemetry: false,
            content_type: WebContentType::Text,
            status_code: 400,
            content: err.into_bytes(),
        }),
    }
}
