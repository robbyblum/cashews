use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use std::{sync::Arc, time::Instant};

use async_once_cell::Lazy;
use futures::FutureExt;
use moka::ops::compute::Op;
use tokio::sync::{OnceCell, Semaphore};

type AsyncCallback<K, V, C> =
    Box<dyn FnMut(K, C) -> Pin<Box<dyn Future<Output = anyhow::Result<V>> + Send>> + Send + Sync>;

#[derive(Clone)]
// todo: maybe take some kind of context variable in?
pub struct SwrCache2<
    K: Hash + Eq + Send + Sync + ToOwned + 'static,
    V: Send + Sync + Clone + 'static,
    C: Clone,
> {
    starter: Arc<Mutex<AsyncCallback<K, V, C>>>,
    inner: moka::future::Cache<K, CachedValue2<V>>,
    time_to_stale: Duration,
    context: Arc<OnceLock<C>>,
}
impl<
    K: Hash + Eq + Send + Sync + ToOwned<Owned = K> + 'static,
    V: Send + Sync + Clone + 'static,
    C: Clone,
> SwrCache2<K, V, C>
{
    pub fn new<F, Fut>(time_to_stale: Duration, max_capacity: u64, f: F) -> SwrCache2<K, V, C>
    where
        F: Fn(K, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    {
        SwrCache2 {
            inner: moka::future::CacheBuilder::new(max_capacity).build(),
            time_to_stale,
            starter: Arc::new(Mutex::new(Box::new(move |x, ctx| Box::pin(f(x, ctx))))),
            context: Arc::new(OnceLock::new()),
        }
    }

    pub fn set_context(&self, context: C) {
        self.context
            .set(context)
            .map_err(|_| ())
            .expect("context was already set");
    }

    fn make_and_spawn_lazy(&self, key: K) -> Arc<InnerValueLazy<V>> {
        let mut _guard = self.starter.lock().unwrap();

        let ctx_clone = self
            .context
            .get()
            .expect("context was not initialized")
            .clone();
        let init_fut = _guard(key, ctx_clone);

        let tts = self.time_to_stale;
        let lazy = Arc::new(Lazy::new(
            (async move {
                let res = init_fut.await.map(|x| CachedValueInner {
                    expiry: Instant::now() + tts,
                    value: Arc::new(x),
                });
                res
            })
            .boxed(),
        ));

        let lazy_cloned = Arc::clone(&lazy);
        tracing::info!("spawning wait task");
        tokio::spawn(async move {
            lazy_cloned.get_unpin().await;
            tracing::info!("wait task finished");
        });

        lazy
    }

    pub async fn get(&self, key: K) -> anyhow::Result<Arc<V>> {
        let owned_key = key.to_owned();
        let entry = self
            .inner
            .entry(key)
            .and_compute_with(|x| async move {
                if let Some(x) = x {
                    let x = x.into_value();

                    match x.value.try_get() {
                        Some(Ok(curr)) => {
                            tracing::info!("curr is Some(Ok(_))");

                            // if value is stale, start the next timer
                            if curr.expiry < Instant::now() && x.next.is_none() {
                                tracing::info!("spawning revalidate");
                                return Op::Put(CachedValue2 {
                                    value: x.value,
                                    next: Some(self.make_and_spawn_lazy(owned_key)),
                                });
                            }
                        }
                        Some(Err(e)) => {
                            // getting current value failed, retry
                            tracing::error!("error refreshing cached value: {:?}", e);
                            return Op::Put(CachedValue2 {
                                value: self.make_and_spawn_lazy(owned_key),
                                next: None,
                            });
                        }
                        None => {
                            tracing::info!("curr is None");
                            // value isn't ready, don't do anything yet
                        }
                    }

                    // error handling for next
                    if let Some(next) = x.next {
                        tracing::info!("next is present");
                        match next.try_get() {
                            Some(Ok(_)) => {
                                tracing::info!("next is Some(Ok(_))");
                                // next is done, swap?
                                return Op::Put(CachedValue2 {
                                    value: next,
                                    next: None,
                                });
                            }
                            Some(Err(e)) => {
                                tracing::error!("error refreshing cached value: {:?}", e);
                                return Op::Put(CachedValue2 {
                                    value: x.value,
                                    next: Some(self.make_and_spawn_lazy(owned_key)),
                                });
                            }
                            None => {
                                tracing::info!("next is None");
                                // value isn't ready, don't do anything yet
                            }
                        }
                    }
                    Op::Nop
                } else {
                    // no value, set it to start
                    Op::Put(CachedValue2 {
                        value: self.make_and_spawn_lazy(owned_key),
                        next: None,
                    })
                }
            })
            .await;

        // and wait for as long as we need...
        let value = entry.into_entry().unwrap().into_value();
        let v = value
            .value
            .get_unpin()
            .await
            .as_ref()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(Arc::clone(&v.value))
    }
}

#[derive(Clone)]
struct CachedValueInner<V> {
    expiry: Instant,
    value: Arc<V>,
}

type InnerValueLazy<V> = Lazy<
    anyhow::Result<CachedValueInner<V>>,
    Pin<Box<dyn Future<Output = anyhow::Result<CachedValueInner<V>>> + Send>>,
>;
#[derive(Clone)]
struct CachedValue2<V> {
    value: Arc<InnerValueLazy<V>>,
    next: Option<Arc<InnerValueLazy<V>>>,
}

#[derive(Clone)]
enum CachedValue<T> {
    // this key has not been queried yet
    // readers will *block* until the future has resolved
    Empty {
        semaphore: Arc<Semaphore>,
        value: Arc<OnceCell<Arc<T>>>,
    },

    // valid result, key not expired yet
    Active {
        value: Arc<T>,
        expiry: Instant,
    },

    // valid result, key *has* expired
    // the semaphore will track
    Stale {
        value: Arc<T>,
        semaphore: Arc<Semaphore>,
    },
}

#[derive(Clone)]
// TODO: can we make this
pub struct SwrCache<
    K: Hash + Eq + Send + Sync + ToOwned + 'static,
    V: Send + Sync + Clone + 'static,
> {
    inner: moka::future::Cache<K, CachedValue<V>>,
    time_to_stale: Duration,
}

impl<K: Hash + Eq + Send + Sync + ToOwned<Owned = K> + 'static, V: Send + Sync + Clone + 'static>
    SwrCache<K, V>
{
    pub fn new(time_to_stale: Duration, max_capacity: u64) -> SwrCache<K, V> {
        SwrCache {
            inner: moka::future::CacheBuilder::new(max_capacity).build(),
            time_to_stale,
        }
    }

    pub async fn get<'a, F, Fut>(&'a self, key: K, getter: F) -> anyhow::Result<Arc<V>>
    where
        F: (FnOnce(K) -> Fut) + Send + 'static,
        Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    {
        let tts = self.time_to_stale;
        let entry = self
            .inner
            .entry_by_ref(&key)
            .and_compute_with(|e| async move {
                if let Some(e) = e {
                    match *e.value() {
                        CachedValue::Active {
                            ref value,
                            ref expiry,
                        } => {
                            if *expiry < Instant::now() {
                                Op::Put(CachedValue::Stale {
                                    value: Arc::clone(value),
                                    semaphore: Arc::new(Semaphore::new(1)),
                                })
                            } else {
                                Op::Nop
                            }
                        }
                        _ => Op::Nop,
                    }
                } else {
                    Op::Put(CachedValue::Empty {
                        semaphore: Arc::new(Semaphore::new(1)),
                        value: Arc::new(OnceCell::new()),
                    })
                }
            })
            .await
            .into_entry()
            .unwrap()
            .into_value();

        let refresh_inner = async |cache: moka::future::Cache<K, CachedValue<V>>,
                                   time_to_stale: Duration,
                                   key: K,
                                   getter: F| {
            let value = Arc::new(getter(key.to_owned()).await?);

            let expiry = Instant::now() + time_to_stale;
            cache
                .insert(
                    key,
                    CachedValue::Active {
                        value: Arc::clone(&value),
                        expiry,
                    },
                )
                .await;

            Ok(value)
        };

        match entry {
            CachedValue::Active { ref value, .. } => Ok(Arc::clone(value)),

            CachedValue::Empty {
                ref semaphore,
                ref value,
            } => {
                let _permit = Arc::clone(&semaphore).acquire_owned().await?;
                // by the time we get this permit, it may have resolved the oncecell
                if let Some(v) = value.get() {
                    Ok(Arc::clone(v))
                } else {
                    let val = Arc::clone(&value);

                    let tts = self.time_to_stale;
                    let cloned_inner = self.inner.clone();
                    let jh = tokio::spawn(async move {
                        let res: anyhow::Result<Arc<V>> = val
                            .get_or_try_init(|| refresh_inner(cloned_inner, tts, key, getter))
                            .await
                            .cloned();

                        drop(_permit);
                        res
                    });
                    let res = jh.await??;
                    Ok(res)
                }
            }

            CachedValue::Stale {
                ref value,
                ref semaphore,
            } => {
                if let Ok(_permit) = Arc::clone(semaphore).try_acquire_owned() {
                    let cloned_inner = self.inner.clone();
                    tokio::spawn(async move {
                        if let Err(e) = refresh_inner(cloned_inner, tts, key, getter).await {
                            tracing::error!("error refreshing cache: {:?}", e);
                        }

                        drop(_permit);
                    });
                }

                Ok(Arc::clone(value))
            }
        }
    }
}
