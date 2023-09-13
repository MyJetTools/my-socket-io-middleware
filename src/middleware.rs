use std::{sync::Arc, time::Duration};

use hyper::Method;
use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpServerMiddleware,
    HttpServerRequestFlow, RequestData, WebContentType,
};
use socket_io_utils::SocketIoSettings;
use tokio::sync::Mutex;

use crate::{
    namespaces::SocketIoNameSpaces, MySocketIo, MySocketIoConnectionsCallbacks, SocketIoList,
    WebSocketCallbacks,
};

pub struct MySocketIoEngineMiddleware {
    pub path_prefix: String,
    socket_id: Mutex<i64>,
    web_socket_callback: Arc<WebSocketCallbacks>,
    socket_io_list: Arc<SocketIoList>,
    registered_sockets: Arc<SocketIoNameSpaces>,
    connections_callback: Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    pub settings: Arc<SocketIoSettings>,
    disconnect_timeout: Duration,
}

impl MySocketIoEngineMiddleware {
    pub fn new(
        connections_callback: Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    ) -> Self {
        let registered_sockets = Arc::new(SocketIoNameSpaces::new());
        let socket_io_list = Arc::new(SocketIoList::new());
        let settings = Arc::new(SocketIoSettings::default());
        Self {
            socket_io_list: socket_io_list.clone(),

            path_prefix: "/socket.io/".to_string(),
            web_socket_callback: Arc::new(WebSocketCallbacks {
                socket_io_list,
                registered_sockets: registered_sockets.clone(),
                connections_callback: connections_callback.clone(),
                settings: settings.clone(),
            }),
            socket_id: Mutex::new(0),
            registered_sockets,
            connections_callback,
            settings,
            disconnect_timeout: Duration::from_secs(60),
        }
    }

    pub async fn register_socket_io(&self, socket_io: Arc<dyn MySocketIo + Send + Sync + 'static>) {
        self.registered_sockets.add(socket_io).await;
    }

    async fn get_socket_id(&self) -> i64 {
        let mut socket_no = self.socket_id.lock().await;
        *socket_no += 1;
        *socket_no
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for MySocketIoEngineMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        if ctx.request.get_path() != self.path_prefix.as_str() {
            return get_next.next(ctx).await;
        }

        if ctx
            .request
            .get_optional_header("sec-websocket-key")
            .is_some()
        {
            if let RequestData::AsRaw(request) = &mut ctx.request.req {
                let id = self.get_socket_id().await;
                return my_http_server_web_sockets::handle_web_socket_upgrade(
                    request,
                    self.web_socket_callback.clone(),
                    id,
                    ctx.request.addr,
                    self.disconnect_timeout,
                )
                .await;
            }

            return get_next.next(ctx).await;
        }

        if ctx.request.method == Method::GET {
            if let Some(result) = handle_get_request(
                ctx,
                &self.connections_callback,
                &self.socket_io_list,
                &self.settings,
            )
            .await
            {
                return result;
            }
        }

        if ctx.request.method == Method::POST {
            return handle_post_request(ctx);
        }

        get_next.next(ctx).await
    }
}

async fn handle_get_request(
    ctx: &mut HttpContext,
    connections_callback: &Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    socket_io_list: &Arc<SocketIoList>,
    settings: &Arc<SocketIoSettings>,
) -> Option<Result<HttpOkResult, HttpFailResult>> {
    let query = ctx.request.get_query_string();

    let query = if let Err(fail_result) = query {
        return Some(Err(fail_result));
    } else {
        query.unwrap()
    };

    let sid = query.get_optional("sid");

    if let Some(sid) = sid {
        return Some(
            HttpOutput::Content {
                headers: None,
                content_type: Some(WebContentType::Text),
                content: socket_io_utils::my_socket_io_messages::compile_connect_payload(sid.value),
            }
            .into_ok_result(true)
            .into(),
        );
    } else {
        let (_, result) =
            crate::process_connect(connections_callback, socket_io_list, settings, None).await;

        let result = HttpOutput::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: result.into_bytes(),
        }
        .into_ok_result(true)
        .into();

        Some(result)
    }
}

fn handle_post_request(_ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Content {
        headers: None,
        content_type: Some(WebContentType::Text),
        content: "ok".to_string().into_bytes(),
    }
    .into_ok_result(true)
    .into();
}
