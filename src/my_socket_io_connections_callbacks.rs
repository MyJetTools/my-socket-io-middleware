use std::sync::Arc;

use my_http_server::HttpFailResult;

use crate::MySocketIoConnection;

#[async_trait::async_trait]
pub trait MySocketIoConnectionsCallbacks {
    async fn connected(&self, socket_io: Arc<MySocketIoConnection>) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, socket_io: Arc<MySocketIoConnection>);
}
