use std::collections::HashMap;
use tokio::sync::RwLock;
use async_trait::async_trait;
use super::cache::ResultCache;

pub struct MemoryResultCache<T: Clone + Send + Sync + 'static> {
    inner: RwLock<HashMap<i32, T>>, 
}

impl<T: Clone + Send + Sync + 'static> MemoryResultCache<T> {
    pub fn new() -> Self { Self { inner: RwLock::new(HashMap::new()) } }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static> ResultCache<T> for MemoryResultCache<T> {
    async fn get(&self, uid: i32) -> Option<T> {
        self.inner.read().await.get(&uid).cloned()
    }
    async fn set(&self, uid: i32, value: T) {
        self.inner.write().await.insert(uid, value);
    }
    async fn invalidate(&self, uid: i32) {
        self.inner.write().await.remove(&uid);
    }
}
