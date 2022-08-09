use std::sync::Arc;

use hyper::Method;
use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpServerMiddleware,
    HttpServerRequestFlow, RequestData, WebContentType,
};
use tokio::sync::Mutex;

use crate::{MySocketIoConnection, SocketIoList, WebSocketCallbacks};

pub struct MySocketIoMiddleware<TCustomData: Sync + Send + Default + 'static> {
    pub path_prefix: String,
    socket_id: Mutex<i64>,
    web_socket_callback: Arc<WebSocketCallbacks<TCustomData>>,
    socket_io_list: Arc<dyn SocketIoList<TCustomData> + Send + Sync + 'static>,
}

impl<TCustomData: Sync + Send + Default + 'static> MySocketIoMiddleware<TCustomData> {
    pub fn new(socket_io_list: Arc<dyn SocketIoList<TCustomData> + Send + Sync + 'static>) -> Self {
        Self {
            socket_io_list: socket_io_list.clone(),
            path_prefix: "/socket.io/".to_string(),
            web_socket_callback: Arc::new(WebSocketCallbacks { socket_io_list }),
            socket_id: Mutex::new(0),
        }
    }

    async fn get_socket_id(&self) -> i64 {
        let mut socket_no = self.socket_id.lock().await;
        *socket_no += 1;
        *socket_no
    }
}

#[async_trait::async_trait]
impl<TCustomData: Sync + Send + Default + 'static> HttpServerMiddleware
    for MySocketIoMiddleware<TCustomData>
{
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
            if let Some(result) = handle_get_request(ctx, &self.socket_io_list).await {
                return result;
            }
        }

        if ctx.request.method == Method::POST {
            return handle_post_request(ctx);
        }

        get_next.next(ctx).await
    }
}

async fn handle_get_request<TCustomData: Sync + Send + Default + 'static>(
    ctx: &mut HttpContext,
    socket_io_list: &Arc<dyn SocketIoList<TCustomData> + Send + Sync + 'static>,
) -> Option<Result<HttpOkResult, HttpFailResult>> {
    let query = ctx.request.get_query_string();

    let query = if let Err(fail_result) = query {
        return Some(Err(fail_result));
    } else {
        query.unwrap()
    };

    let sid = query.get_optional("sid");

    if let Some(sid) = sid {
        println!("Get with Id:{}", sid.value);

        return Some(
            HttpOutput::Content {
                headers: None,
                content_type: Some(WebContentType::Text),
                content: "5".to_string().into_bytes(),
            }
            .into_ok_result(true)
            .into(),
        );
    } else {
        let sid = uuid::Uuid::new_v4().to_string();

        let sid = sid.replace("-", "")[..8].to_string();
        println!("Create new Id:{}", sid);

        let result = super::models::compile_negotiate_response(sid.as_str());

        let socket_io = MySocketIoConnection::new(sid, TCustomData::default());

        socket_io_list.add(Arc::new(socket_io)).await;

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

fn handle_post_request(ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Content {
        headers: None,
        content_type: Some(WebContentType::Text),
        content: "5".to_string().into_bytes(),
    }
    .into_ok_result(true)
    .into();
}
