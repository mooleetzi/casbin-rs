use crate::cache::Cache;

use async_trait::async_trait;
use ttl_cache::TtlCache;

use std::hash::Hash;
use std::time::Duration;

pub struct DefaultCache<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    pub ttl: Duration,
    cache: TtlCache<K, V>,
}

impl<K, V> DefaultCache<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    pub fn new(cap: usize) -> Box<dyn Cache<K, V>> {
        Box::new(DefaultCache {
            ttl: Duration::from_secs(120),
            cache: TtlCache::new(cap),
        }) as Box<dyn Cache<K, V>>
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for DefaultCache<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    fn set_capacity(&mut self, cap: usize) {
        self.cache.set_capacity(cap);
    }

    fn set_ttl(&mut self, ttl: Duration) {
        self.ttl = ttl;
    }

    async fn get<'a>(&'a self, k: &K) -> Option<&'a V> {
        self.cache.get(k)
    }

    async fn has(&self, k: &K) -> bool {
        self.cache.contains_key(k)
    }

    async fn set(&mut self, k: K, v: V) {
        if self.has(&k).await {
            self.cache.remove(&k);
        }
        self.cache.insert(k, v, self.ttl);
    }

    async fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use std::thread::sleep;

    #[test]
    fn test_set_and_get() {
        let mut cache = DefaultCache::new(1);

        task::block_on(async move {
            cache.set(vec!["alice", "/data1", "read"], false).await;
            assert!(cache.get(&vec!["alice", "/data1", "read"]).await == Some(&false));
        });
    }

    #[test]
    fn test_set_ttl() {
        let mut cache = DefaultCache::new(1);
        cache.set_ttl(Duration::from_secs(2));

        task::block_on(async move {
            cache.set(vec!["alice", "/data1", "read"], false).await;

            sleep(Duration::from_secs(1));
            assert!(cache.get(&vec!["alice", "/data1", "read"]).await == Some(&false));

            sleep(Duration::from_secs(2));
            assert!(!cache.has(&vec!["alice", "/data1", "read"]).await);
        });
    }

    #[test]
    fn test_capacity() {
        let mut cache = DefaultCache::new(1);

        task::block_on(async move {
            cache.set(vec!["alice", "/data1", "read"], false).await;
            cache.set(vec!["bob", "/data2", "write"], false).await;
            assert!(!cache.has(&vec!["alice", "/data1", "read"]).await);
            assert!(cache.has(&vec!["bob", "/data2", "write"]).await);
        });
    }

    #[test]
    fn test_set_capacity() {
        let mut cache = DefaultCache::new(1);
        cache.set_capacity(2);

        task::block_on(async move {
            cache.set(vec!["alice", "/data1", "read"], false).await;
            cache.set(vec!["bob", "/data2", "write"], false).await;
            cache
                .set(vec!["unknow", "/data3", "read_write"], false)
                .await;
            assert!(!cache.has(&vec!["alice", "/data1", "read"]).await);
            assert!(cache.has(&vec!["bob", "/data2", "write"]).await);
            assert!(cache.has(&vec!["unknow", "/data3", "read_write"]).await);
        });
    }
}
