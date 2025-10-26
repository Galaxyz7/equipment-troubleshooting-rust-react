#![allow(dead_code)] // Module is used by library, not directly by binary

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A simple TTL-based in-memory cache
#[derive(Debug, Clone)]
pub struct Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    store: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
    max_size: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new cache with specified TTL and max size
    pub fn new(ttl_seconds: u64, max_size: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
            max_size,
        }
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            if Instant::now() < entry.expires_at {
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Insert a value into the cache
    pub async fn set(&self, key: K, value: V) {
        let mut store = self.store.write().await;

        // Evict oldest entries if cache is full
        if store.len() >= self.max_size {
            // Simple FIFO eviction - remove first entry
            if let Some(first_key) = store.keys().next().cloned() {
                store.remove(&first_key);
            }
        }

        store.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    /// Invalidate (remove) a specific key
    pub async fn invalidate(&self, key: &K) {
        let mut store = self.store.write().await;
        store.remove(key);
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
    }

    /// Remove expired entries
    pub async fn cleanup(&self) {
        let mut store = self.store.write().await;
        let now = Instant::now();
        store.retain(|_, entry| now < entry.expires_at);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let store = self.store.read().await;
        let now = Instant::now();
        let active_count = store.values().filter(|e| now < e.expires_at).count();

        CacheStats {
            total_entries: store.len(),
            active_entries: active_count,
            expired_entries: store.len() - active_count,
            max_size: self.max_size,
            ttl_seconds: self.ttl.as_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
    pub max_size: usize,
    pub ttl_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic() {
        let cache = Cache::new(60, 100);

        cache.set("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let cache = Cache::new(1, 100); // 1 second TTL

        cache.set("key".to_string(), "value".to_string()).await;
        assert_eq!(cache.get(&"key".to_string()).await, Some("value".to_string()));

        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(cache.get(&"key".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = Cache::new(60, 100);

        cache.set("key".to_string(), "value".to_string()).await;
        assert!(cache.get(&"key".to_string()).await.is_some());

        cache.invalidate(&"key".to_string()).await;
        assert!(cache.get(&"key".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_max_size() {
        let cache = Cache::new(60, 2);

        cache.set(1, "a").await;
        cache.set(2, "b").await;
        cache.set(3, "c").await; // Should evict oldest

        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 2);
    }
}
