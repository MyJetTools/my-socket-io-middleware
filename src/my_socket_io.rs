#[async_trait::async_trait]
pub trait MySocketIo {
    async fn on(&self, event_id: &str, message: &str) -> Option<String>;
}
