use std::sync::Arc;

use crate::MySocketIoConnection;

#[async_trait::async_trait]
pub trait SocketIoList<TCustomData: Sync + Send + 'static> {
    async fn add(&self, socket_io_connection: Arc<MySocketIoConnection<TCustomData>>);
    async fn remove(&self, socket_io_id: &str);
    async fn get(&self, socket_io_id: &str) -> Option<Arc<MySocketIoConnection<TCustomData>>>;
    async fn get_by_web_socket_id(
        &self,
        web_socket_id: i64,
    ) -> Option<Arc<MySocketIoConnection<TCustomData>>>;
}
