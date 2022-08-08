use std::sync::Arc;

use my_http_server::HttpFailResult;

use crate::{MySocketIoConnection, MySocketIoMessage};

#[async_trait::async_trait]
pub trait MySocketIoCallbacks<TCustomData: Send + Sync + Default + 'static> {
    async fn connected(
        &self,
        socket_io: Arc<MySocketIoConnection<TCustomData>>,
    ) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, socket_io: Arc<MySocketIoConnection<TCustomData>>);
    async fn on_message(
        &self,
        socket_io: Arc<MySocketIoConnection<TCustomData>>,
        message: Arc<MySocketIoMessage>,
    );
}
