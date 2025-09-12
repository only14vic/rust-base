use {
    super::{Cache, Cacher},
    actix_web::rt::{spawn, time::sleep},
    app_base::prelude::*,
    dashmap::DashMap,
    std::{
        any::Any,
        future::Future,
        sync::{
            Arc, LazyLock,
            atomic::{AtomicBool, Ordering}
        },
        time::{Duration, Instant, SystemTime, UNIX_EPOCH}
    }
};

const MAINTAINE_TIMEOUT_SECS: u64 = 5;
static DEFAULT_CACHE_CAPACITY: LazyLock<usize> = LazyLock::new(|| {
    option_env!("CACHE_CAPACITY")
        .unwrap_or("1000")
        .parse::<usize>()
        .unwrap()
});

#[inline]
fn now() -> u64 {
    static TIME: LazyLock<Instant> = LazyLock::new(|| {
        Instant::now() - SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    });

    TIME.elapsed().as_secs()
}

#[derive(Debug)]
struct CacheItem {
    data: Arc<dyn Any + Send + Sync>,
    expired: u64
}

type CacheBuffer = DashMap<String, CacheItem>;

#[derive(Clone, Debug)]
pub struct ArrayCache {
    is_maintained: Arc<AtomicBool>,
    buffer: Arc<CacheBuffer>
}

impl Cacher<ArrayCache> {
    pub fn from_static() -> &'static Self {
        static INSTANCE: LazyLock<Cacher<ArrayCache>> = LazyLock::new(|| {
            Cacher::with(ArrayCache {
                is_maintained: Default::default(),
                buffer: Arc::new(DashMap::with_capacity(*DEFAULT_CACHE_CAPACITY))
            })
        });

        if actix_web::rt::System::is_registered()
            && INSTANCE.is_maintained.swap(true, Ordering::SeqCst) == false
        {
            // Maintain array cache
            spawn(async {
                let cacher = Cacher::<ArrayCache>::from_static();

                loop {
                    sleep(Duration::from_secs(MAINTAINE_TIMEOUT_SECS)).await;

                    if let Ok(num) = cacher.remove_expired().await
                        && num > 0
                    {
                        Env::is_debug()
                            .then(|| log::trace!("Removed expired cache items: {num}"));
                    }
                }
            });
        }

        &INSTANCE
    }
}

impl Cache for ArrayCache {
    async fn get<T: Send + Sync + 'static>(
        &self,
        keys: &[&str]
    ) -> OkAsync<Option<Arc<T>>> {
        let key = self.get_key(keys);

        match self.buffer.get(&key) {
            Some(v) if v.expired > now() => {
                v.data
                    .clone()
                    .downcast::<T>()
                    .map_err(|_| "Invalid downcast type.")?
                    .into_ok()
            },
            _ => Ok(None)
        }
    }

    // Returns old value if exists.
    async fn set<T: Send + Sync + 'static>(
        &self,
        keys: &[&str],
        value: T,
        lifetime: u64
    ) -> OkAsync<Option<Arc<T>>> {
        let key = self.get_key(keys);

        self.buffer
            .insert(
                key,
                CacheItem {
                    data: Arc::new(value) as Arc<dyn Any + Send + Sync>,
                    expired: now() + lifetime
                }
            )
            .map(|v| v.data.downcast().map_err(|_| "Invalid downcast type."))
            .transpose()?
            .into_ok()
    }

    async fn exists(&self, keys: &[&str]) -> OkAsync<bool> {
        let key = self.get_key(keys);

        self.buffer.contains_key(&key).into_ok()
    }

    async fn len(&self) -> usize {
        self.buffer.len()
    }

    async fn keys(&self) -> Vec<String> {
        self.buffer.iter().map(|v| v.key().to_string()).collect()
    }

    async fn remove(&self, keys: &[&str]) -> VoidAsync {
        let key = self.get_key(keys);

        self.buffer.remove(&key);

        ok()
    }

    async fn remove_expired(&self) -> OkAsync<usize> {
        let now = now();
        let mut num = 0;

        self.buffer.retain(|_, v| {
            let is_live = v.expired > now;
            if is_live == false {
                num += 1;
            }
            is_live
        });

        Ok(num)
    }

    async fn clear(&self, keys: &[&str]) -> VoidAsync {
        let key = self.get_key(keys);

        if key.is_empty() {
            self.clear_all().await?;
        } else {
            self.buffer.retain(|k, _| k.starts_with(&key) == false);
        }

        ok()
    }

    async fn clear_all(&self) -> VoidAsync {
        self.buffer.clear();
        ok()
    }

    async fn getset<T: Send + Sync + 'static>(
        &self,
        keys: &[&str],
        lifetime: u64,
        callback: impl Future<Output = OkAsync<T>>
    ) -> OkAsync<Option<Arc<T>>> {
        match self.get(keys).await? {
            Some(v) => Ok(Some(v)),
            None => {
                let value = callback.await?;
                let _ = self.set(keys, value, lifetime).await?;
                self.get(keys).await
            }
        }
    }
}
