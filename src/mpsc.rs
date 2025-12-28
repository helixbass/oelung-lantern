use async_trait::async_trait;

#[async_trait]
pub trait Sender<TValue>: Send + Sync {
    async fn send(&self, value: TValue);
}
