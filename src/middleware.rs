use std::{collections::HashMap, sync::Arc};

use hyper::Method;
use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpServerMiddleware,
    HttpServerRequestFlow, RequestData, WebContentType,
};
use tokio::sync::Mutex;

use crate::{
    MySocketIo, MySocketIoConnection, MySocketIoConnectionsCallbacks, SocketIoList,
    SocketIoSettings, WebSocketCallbacks,
};

pub struct MySocketIoEngineMiddleware {
    pub path_prefix: String,
    socket_id: Mutex<i64>,
    web_socket_callback: Arc<WebSocketCallbacks>,
    socket_io_list: Arc<SocketIoList>,
    registered_sockets: Arc<Mutex<HashMap<String, Arc<dyn MySocketIo + Send + Sync + 'static>>>>,
    connections_callback: Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    pub settings: Arc<SocketIoSettings>,
}

impl MySocketIoEngineMiddleware {
    pub fn new(
        connections_callback: Arc<dyn MySocketIoConnectionsCallbacks + Send + Sync + 'static>,
    ) -> Self {
        let registered_sockets = Arc::new(Mutex::new(HashMap::new()));
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
        }
    }

    pub async fn register_socket_io(
        &self,
        namespace: String,
        socket_io: Arc<dyn MySocketIo + Send + Sync + 'static>,
    ) {
        let mut write_access = self.registered_sockets.lock().await;
        write_access.insert(namespace, socket_io);
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
        if ctx.request.get_path_lower_case() != self.path_prefix.as_str() {
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
                    &self.web_socket_callback,
                    id,
                    ctx.request.addr,
                )
                .await;
            }

            return get_next.next(ctx).await;
        }

        if ctx.request.method == Method::GET {
            if let Some(result) =
                handle_get_request(ctx, &self.connections_callback, &self.socket_io_list).await
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
) -> Option<Result<HttpOkResult, HttpFailResult>> {
    let query = ctx.request.get_query_string();

    let query = if let Err(fail_result) = query {
        return Some(Err(fail_result));
    } else {
        query.unwrap()
    };

    let sid = query.get_optional("sid");

    if let Some(sid) = sid {
        let mut content = Vec::new();
        content.extend_from_slice("40{\"sid\":\"".as_bytes());
        content.extend_from_slice(sid.value.as_bytes());
        content.extend_from_slice("\"}".as_bytes());

        return Some(
            HttpOutput::Content {
                headers: None,
                content_type: Some(WebContentType::Text),
                content,
            }
            .into_ok_result(true)
            .into(),
        );
    } else {
        let sid = uuid::Uuid::new_v4().to_string();

        let sid = sid.replace("-", "")[..8].to_string();

        let result = super::models::compile_negotiate_response(sid.as_str());

        let socket_io = MySocketIoConnection::new(sid);
        let socket_io_connection = Arc::new(socket_io);

        connections_callback
            .connected(socket_io_connection.clone())
            .await
            .unwrap();

        socket_io_list.add_socket_io(socket_io_connection).await;

        let result = HttpOutput::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: result.to_owned().into_bytes(),
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
