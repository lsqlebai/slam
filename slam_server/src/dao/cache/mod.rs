use async_trait::async_trait;

#[async_trait]
pub trait ResultCache<T: Clone + Send + Sync + 'static, K: Clone + Send + Sync + 'static> {
    async fn get(&self, key: K) -> Option<T>;
    async fn set(&self, key: K, value: T);
    async fn invalidate(&self, key: K);
}

pub mod memory;

