use std::collections::HashMap;
use tokio::sync::RwLock;
use async_trait::async_trait;
use super::cache::ResultCache;
use std::hash::Hash;

pub struct MemoryResultCache<T: Clone + Send + Sync + 'static, K: Eq + Hash + Clone + Send + Sync + 'static> {
    inner: RwLock<HashMap<K, T>>, 
}

impl<T: Clone + Send + Sync + 'static, K: Eq + Hash + Clone + Send + Sync + 'static> MemoryResultCache<T, K> {
    pub fn new() -> Self { Self { inner: RwLock::new(HashMap::new()) } }
}

impl<T: Clone + Send + Sync + 'static, K: Eq + Hash + Clone + Send + Sync + 'static> Default for MemoryResultCache<T, K> {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static, K: Eq + Hash + Clone + Send + Sync + 'static> ResultCache<T, K> for MemoryResultCache<T, K> {
    async fn get(&self, key: K) -> Option<T> {
        self.inner.read().await.get(&key).cloned()
    }
    async fn set(&self, key: K, value: T) {
        self.inner.write().await.insert(key, value);
    }
    async fn invalidate(&self, key: K) {
        self.inner.write().await.remove(&key);
    }
}
