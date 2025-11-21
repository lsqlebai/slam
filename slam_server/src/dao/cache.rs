use async_trait::async_trait;

#[async_trait]
pub trait ResultCache<T: Clone + Send + Sync + 'static> {
    async fn get(&self, uid: i32) -> Option<T>;
    async fn set(&self, uid: i32, value: T);
    async fn invalidate(&self, uid: i32);
}
